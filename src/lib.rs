#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
use serde::{Deserialize, Serialize};
use vetis::{
    VetisHook, VetisResult,
    errors::{ConfigError, VetisError},
    hook::HookConfig,
    matcher::MatcherConfig,
};

use crate::tokio::ReverseProxyHook;

#[cfg(feature = "runtime-tokio")]
/// Tokio runtime support module
pub mod tokio;

#[cfg(test)]
mod tests;

/// Builder for creating `ProxyHookConfig` instances.
#[derive(Deserialize)]
pub struct ReverseProxyHookConfigBuilder {
    matcher: Option<Box<dyn MatcherConfig>>,
    target: String,
}

impl ReverseProxyHookConfigBuilder {
    /// Allow set a matcher
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn matcher<M>(mut self, matcher: M) -> Self
    where
        M: MatcherConfig + Send + Sync + 'static,
    {
        self.matcher = Some(Box::new(matcher));
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

    /// Build the `ReverseProxyHookConfig` with the configured settings.
    ///
    /// # Returns
    ///
    /// * `VetisResult<ReverseProxyHookConfig>` - The `ReverseProxyHookConfig` with the configured settings.
    pub fn build(self) -> VetisResult<ReverseProxyHookConfig> {
        if self
            .target
            .is_empty()
        {
            return Err(VetisError::Config(ConfigError::hook("Target cannot be empty")));
        }

        Ok(ReverseProxyHookConfig {
            matcher: self
                .matcher
                .map_or(None, |val| Some(val)),
            target: self.target,
        })
    }
}

/// Configuration for a proxy path.
#[derive(Clone, Serialize, Deserialize)]
pub struct ReverseProxyHookConfig {
    matcher: Option<Box<dyn MatcherConfig>>,
    target: String,
}

#[typetag::serde(name = "rev-proxy")]
impl HookConfig for ReverseProxyHookConfig {
    fn matcher(&self) -> Option<&Box<dyn MatcherConfig>> {
        self.matcher
            .as_ref()
    }

    fn boxed_clone(&self) -> Box<dyn HookConfig> {
        Box::new(self.clone())
    }

    fn boxed_hook(&self) -> Box<dyn VetisHook> {
        Box::new(ReverseProxyHook::new(self.clone()))
    }
}

impl ReverseProxyHookConfig {
    /// Creates a new `ProxyHookConfigBuilder` with default settings.
    ///
    /// # Returns
    ///
    /// * `ProxyHookConfigBuilder` - The builder.
    pub fn builder() -> ReverseProxyHookConfigBuilder {
        ReverseProxyHookConfigBuilder { matcher: None, target: "http://localhost:8080".to_string() }
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
