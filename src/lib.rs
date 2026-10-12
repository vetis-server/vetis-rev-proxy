#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
use serde::{Deserialize, Serialize};
use vetis::{
    VetisResult, errors::{ConfigError, VetisError}, host::path::{Path, PathConfig},
};

use crate::tokio::ReveseProxyPath;

#[cfg(feature = "runtime-tokio")]
/// Tokio runtime support module
pub mod tokio;

#[cfg(test)]
mod tests;

/// Builder for creating `ProxyPathConfig` instances.
#[derive(Deserialize)]
pub struct ReverseProxyPathConfigBuilder {
    uri: String,
    target: String,
}

impl ReverseProxyPathConfigBuilder {
    /// Allow set the URI of the proxy path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn uri(mut self, uri: &str) -> Self {
        self.uri = uri.to_string();
        self
    }

    /// Allow set the target of the proxy path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn target(mut self, target: &str) -> Self {
        self.target = target.to_string();
        self
    }

    /// Build the `ReverseProxyPathConfig` with the configured settings.
    ///
    /// # Returns
    ///
    /// * `VetisResult<ReverseProxyPathConfig>` - The `ReverseProxyPathConfig` with the configured settings.
    pub fn build(self) -> VetisResult<ReverseProxyPathConfig> {
        if self.uri.is_empty() {
            return Err(VetisError::Config(ConfigError::Path("URI cannot be empty".to_string())));
        }
        if self
            .target
            .is_empty()
        {
            return Err(VetisError::Config(ConfigError::Path(
                "Target cannot be empty".to_string(),
            )));
        }

        Ok(ReverseProxyPathConfig { uri: self.uri, target: self.target })
    }
}

/// Configuration for a proxy path.
#[derive(Clone, Serialize, Deserialize)]
pub struct ReverseProxyPathConfig {
    uri: String,
    target: String,
    // TODO: Add custom proxy rules

    // TODO: Add support for custom headers
}

#[typetag::serde(name = "rev-proxy")]
impl PathConfig for ReverseProxyPathConfig {
    fn uri(&mut self, uri: &str) {
        self.uri = uri.to_string();
    }

    fn boxed_clone(&self) -> Box<dyn PathConfig> {
        Box::new(self.clone())
    }

    fn boxed_path(&self) -> Box<dyn Path> {
        Box::new(ReveseProxyPath::new(self.clone()))
    }

    fn boxed_sync(&self) -> Box<dyn Path + Send + Sync> {
        Box::new(ReveseProxyPath::new(self.clone()))
    }
}

impl ReverseProxyPathConfig {
    /// Creates a new `ProxyPathConfigBuilder` with default settings.
    ///
    /// # Returns
    ///
    /// * `ProxyPathConfigBuilder` - The builder.
    pub fn builder() -> ReverseProxyPathConfigBuilder {
        ReverseProxyPathConfigBuilder {
            uri: "/test".to_string(),
            target: "http://localhost:8080".to_string(),
        }
    }

    /// Returns the target of the proxy path.
    ///
    /// # Returns
    ///
    /// * `&str` - The target of the proxy path.
    pub fn target(&self) -> &str {
        &self.target
    }
}
