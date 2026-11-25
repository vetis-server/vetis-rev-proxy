use std::net::Ipv4Addr;

use caramelo::{expect, matchers::eq};
use deboa::{
    cert::{CertificateExt, ContentEncoding},
    request,
};
use deboa_tokio::cert::DeboaCertificate;
use http::{StatusCode, Version};
use tokio::fs;
use vetis::{VetisTestResult, host::HostConfig};
use vetis_rev_proxy::ReverseProxyHookConfig;
use vetis_tokio::{
    Response, TlsConfig, Vetis, VetisServer as _,
    hook::{HandlerHookConfig, handler_fn},
};

use crate::common::{CA_CERT, SERVER_CERT, SERVER_KEY};

#[tokio::test]
async fn test_get_proxy_to_target() -> VetisTestResult<()> {
    let tls_config = TlsConfig::builder()
        .ca_file(CA_CERT)
        .cert_file(SERVER_CERT)
        .key_file(SERVER_KEY)
        .build()?;

    let source_host_config = HostConfig::builder()
        .hostname("localhost")
        .root_directory(".")
        .tls(tls_config)
        .add_proto(Version::HTTP_11)
        .add_bind_address(Ipv4Addr::UNSPECIFIED, 8084)
        .add_hook(
            ReverseProxyHookConfig::builder()
                .target("http://localhost:8085")
                .build()?,
        )
        .build()?;

    let target_host_config = HostConfig::builder()
        .hostname("localhost")
        .root_directory(".")
        .add_bind_address(Ipv4Addr::UNSPECIFIED, 8085)
        .add_hook(
            HandlerHookConfig::builder()
                .handler(handler_fn(|_request| async move {
                    let response = Response::builder()
                        .status(StatusCode::OK)
                        .text("Hello, world!");
                    Ok(response)
                }))
                .build()?,
        )
        .build()?;

    assert_eq!(target_host_config.hostname(), "localhost");

    let mut server = Vetis::builder()
        .add_host(source_host_config)
        .add_host(target_host_config)
        .build();

    server
        .start()
        .await?;

    let cert = fs::read(CA_CERT).await?;
    let client = deboa_tokio::Client::builder()
        .certificate(DeboaCertificate::from_slice(&cert, ContentEncoding::DER))
        .prior_knowledge(true)
        .build();

    let request = request::get("https://localhost:8084/")?
        .version(Version::HTTP_11)
        .send_with(&client)
        .await?;

    expect(request.status()).to_be(eq(StatusCode::OK));
    expect(
        request
            .text()
            .await?,
    )
    .to_be(eq("Hello, world!"));

    server
        .stop()
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_post_proxy_to_target() -> VetisTestResult<()> {
    let tls_config = TlsConfig::builder()
        .ca_file(CA_CERT)
        .cert_file(SERVER_CERT)
        .key_file(SERVER_KEY)
        .build()?;

    let source_host_config = HostConfig::builder()
        .hostname("localhost")
        .root_directory(".")
        .tls(tls_config.clone())
        .add_proto(Version::HTTP_11)
        .add_bind_address(Ipv4Addr::UNSPECIFIED, 9093)
        .add_hook(
            ReverseProxyHookConfig::builder()
                .target("http://localhost:9094")
                .build()?,
        )
        .build()?;

    let target_host_config = HostConfig::builder()
        .hostname("localhost")
        .root_directory(".")
        .add_bind_address(Ipv4Addr::UNSPECIFIED, 9094)
        .add_hook(
            HandlerHookConfig::builder()
                .handler(handler_fn(|request| async move {
                    let (_, body) = request.into_parts();
                    let response = Response::builder()
                        .status(StatusCode::OK)
                        .body(body);

                    Ok(response)
                }))
                .build()?,
        )
        .build()?;

    assert_eq!(target_host_config.hostname(), "localhost");

    let mut server = Vetis::builder()
        .add_host(source_host_config)
        .add_host(target_host_config)
        .build();

    server
        .start()
        .await?;

    let cert = fs::read(CA_CERT).await?;
    let client = deboa_tokio::Client::builder()
        .certificate(DeboaCertificate::from_slice(&cert, ContentEncoding::DER))
        .prior_knowledge(true)
        .build();

    let response = request::post("https://localhost:9093/")?
        .text("Something cool!")
        .version(Version::HTTP_11)
        .send_with(&client)
        .await?;

    expect(response.status()).to_be(eq(StatusCode::OK));
    expect(
        response
            .text()
            .await?,
    )
    .to_be(eq("Something cool!"));

    server
        .stop()
        .await?;

    Ok(())
}
