use crate::ReverseProxyHookConfig;
use cel::Context;
use deboa_tokio::{Client, HttpClient as _, client::HttpConnectionPool, request::DeboaRequest};

use http::StatusCode;
use once_cell::sync::Lazy;
use vetis::{
    HookPhases, Request, Response, VetisFutureResult,
    errors::{HostError, RuntimeError, VetisError},
    hook::{Hook, HookConfig as _, HookPhase},
    make_bitflags,
    matcher::Matcher,
};

static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .connection_pool(HttpConnectionPool::default())
        .prior_knowledge(true)
        .build()
});

/// Proxy path
pub struct ReverseProxyHook {
    matcher: Option<Box<dyn Matcher>>,
    config: ReverseProxyHookConfig,
}

unsafe impl Send for ReverseProxyHook {}
unsafe impl Sync for ReverseProxyHook {}

impl ReverseProxyHook {
    /// Create a new proxy path with provided configuration
    ///
    /// # Arguments
    ///
    /// * `config` - The proxy path configuration
    ///
    /// # Returns
    ///
    /// * `ProxyHook` - The proxy path
    pub fn new(config: ReverseProxyHookConfig) -> ReverseProxyHook {
        let matcher = if let Some(matcher_config) = config.matcher() {
            Some(matcher_config.boxed_matcher())
        } else {
            None
        };

        ReverseProxyHook { matcher, config }
    }
}

impl Hook for ReverseProxyHook {
    fn has_matches(&self, context: &Context) -> bool {
        if let Some(matcher) = &self.matcher { matcher.is_match(context) } else { true }
    }

    fn phases(&self) -> HookPhases {
        make_bitflags!(HookPhase::Access)
    }

    fn matcher_mut(&mut self) -> Option<&mut Box<dyn Matcher>> {
        self.matcher
            .as_mut()
    }

    fn handle<'a>(
        &'a self,
        request: Request,
        _response: Option<Response>,
    ) -> VetisFutureResult<'a, (Request, Option<Response>)> {
        let future = async move {
            let target = self.config.target();

            let (request_parts, request_body) = request.into_parts();

            let target_url = format!(
                "{}{}",
                target,
                request_parts
                    .uri
                    .path()
            );
            let deboa_request = match DeboaRequest::at(
                target_url,
                request_parts
                    .method
                    .clone(),
            ) {
                Ok(request) => request,
                Err(e) => {
                    return Err(VetisError::Host(HostError::Runtime(
                        RuntimeError::new(&e.to_string()).with_status_code(StatusCode::BAD_GATEWAY),
                    )));
                }
            };

            let deboa_request = match deboa_request
                .version(request_parts.version)
                .headers(
                    request_parts
                        .headers
                        .clone(),
                )
                .body(request_body)
                .build()
            {
                Ok(request) => request,
                Err(e) => {
                    return Err(VetisError::Host(HostError::Runtime(
                        RuntimeError::new(&e.to_string()).with_status_code(StatusCode::BAD_GATEWAY),
                    )));
                }
            };

            // TODO: Check errors and handle them properly by returning a proper response 500, 503 or 504
            let upstream_response = CLIENT
                .execute(deboa_request)
                .await;

            let upstream_response = match upstream_response {
                Ok(response) => response,
                Err(e) => {
                    return Err(VetisError::Host(HostError::Runtime(
                        RuntimeError::new(&e.to_string()).with_status_code(StatusCode::BAD_GATEWAY),
                    )));
                }
            };

            let (response_parts, response_body) = upstream_response.into_parts();

            let request =
                Request::from_parts(request_parts.clone(), hyper_body_utils::HttpBody::empty());

            let response = Some(
                Response::builder()
                    .status(response_parts.status)
                    .headers(response_parts.headers)
                    .body(response_body),
            );

            Ok((request, response))
        };

        Box::pin(future)
    }
}
