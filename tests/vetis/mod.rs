use caramelo::{expect, matchers::eq};
use deboa::{
    cert::{CertificateExt, ContentEncoding},
    request,
};
use deboa_tokio::cert::DeboaCertificate;
use http::{StatusCode, Version};
use tokio::fs;
use vetis::{
    VetisTestResult,
    host::{Host as _, HostConfig},
};
use vetis_rev_proxy::{ReverseProxyHookConfig, tokio::ReverseProxyHook};
use vetis_tokio::{
    Response, TlsConfig, Vetis, VetisServer as _,
    hook::{HandlerHook, handler_fn},
    host::Host,
};

use crate::common::{CA_CERT, SERVER_CERT, SERVER_KEY};

#[tokio::test]
async fn test_get_proxy_to_target() -> VetisTestResult<()> {
    let security_config = TlsConfig::builder()
        .ca_file(CA_CERT)
        .cert_file(SERVER_CERT)
        .key_file(SERVER_KEY)
        .build()?;

    let source_host_config = HostConfig::builder()
        .hostname("localhost")
        .root_directory(".")
        .protos(&[Version::HTTP_11])
        .tls(security_config)
        .bind_addresses(&[(
            "0.0.0.0"
                .parse()
                .unwrap(),
            8084,
        )])
        .build()?;

    let mut source_host = Host::new(source_host_config).await?;
    source_host.add_hook(ReverseProxyHook::new(
        ReverseProxyHookConfig::builder()
            .target("http://localhost:8085")
            .build()?,
    ));

    let target_host_config = HostConfig::builder()
        .hostname("localhost")
        .root_directory(".")
        .bind_addresses(&[(
            "0.0.0.0"
                .parse()
                .unwrap(),
            8085,
        )])
        .build()?;

    let mut target_host = Host::new(target_host_config).await?;
    target_host.add_hook(
        HandlerHook::builder()
            .handler(handler_fn(|_request| async move {
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .text("Hello, world!");
                Ok(response)
            }))
            .build()?,
    );

    assert_eq!(
        target_host
            .config()
            .hostname(),
        "localhost"
    );

    let mut server = Vetis::builder()
        .add_host(source_host)
        .await?
        .add_host(target_host)
        .await?
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
    let security_config = TlsConfig::builder()
        .ca_file(CA_CERT)
        .cert_file(SERVER_CERT)
        .key_file(SERVER_KEY)
        .build()?;

    let source_host_config = HostConfig::builder()
        .hostname("localhost")
        .root_directory(".")
        .protos(&[Version::HTTP_11])
        .tls(security_config.clone())
        .bind_addresses(&[(
            "0.0.0.0"
                .parse()
                .unwrap(),
            9093,
        )])
        .build()?;

    let mut source_host = Host::new(source_host_config).await?;
    source_host.add_hook(ReverseProxyHook::new(
        ReverseProxyHookConfig::builder()
            .target("http://localhost:9094")
            .build()?,
    ));

    let target_host_config = HostConfig::builder()
        .hostname("localhost")
        .root_directory(".")
        .bind_addresses(&[(
            "0.0.0.0"
                .parse()
                .unwrap(),
            9094,
        )])
        .build()?;

    let mut target_host = Host::new(target_host_config).await?;
    target_host.add_hook(
        HandlerHook::builder()
            .handler(handler_fn(|request| async move {
                let (_, body) = request.into_parts();
                let response = Response::builder()
                    .status(StatusCode::OK)
                    .body(body);

                Ok(response)
            }))
            .build()?,
    );

    assert_eq!(
        target_host
            .config()
            .hostname(),
        "localhost"
    );

    let mut server = Vetis::builder()
        .add_host(source_host)
        .await?
        .add_host(target_host)
        .await?
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
