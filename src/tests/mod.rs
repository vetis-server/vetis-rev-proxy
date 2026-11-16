use crate::ReverseProxyHookConfig;
use vetis::{
    VetisTestResult,
    errors::{ConfigError, VetisError},
};

#[test]
fn test_reverse_proxy_config() -> VetisTestResult<()> {
    let reverse_proxy_config = ReverseProxyHookConfig::builder()
        .target("http://localhost:8081")
        .build()?;
    assert_eq!(reverse_proxy_config.target(), "http://localhost:8081");
    Ok(())
}

#[test]
fn test_proxy_path() -> VetisTestResult<()> {
    let some_path = ReverseProxyHookConfig::builder()
        .target("http://localhost:8080")
        .build()?;

    assert_eq!(some_path.target(), "http://localhost:8080");

    Ok(())
}

#[test]
fn test_invalid_proxy_path_target() -> VetisTestResult<()> {
    let some_path = ReverseProxyHookConfig::builder()
        .target("")
        .build();

    assert!(some_path.is_err());
    assert_eq!(
        some_path.err(),
        Some(VetisError::Config(ConfigError::hook("Target cannot be empty")))
    );

    Ok(())
}
