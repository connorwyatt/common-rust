use std::time::Duration;

use axum::http::{Request, Response};
use tower_http::trace::{MakeSpan, OnRequest, OnResponse};
use tracing::Span;

use crate::latency::Latency;

#[derive(Clone, Debug, Default)]
pub(crate) struct DefaultMakeSpan {}

impl<B> MakeSpan<B> for DefaultMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        tracing::debug_span!("request",
                        method = %request.method(),
                        uri = %request.uri(),
                        request_id = ?request.headers().get("X-Request-Id").expect("request ID is always set"))
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct DefaultOnRequest {}

impl<B> OnRequest<B> for DefaultOnRequest {
    fn on_request(&mut self, _: &Request<B>, span: &Span) {
        tracing::debug!(parent: span, "started processing request");
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct DefaultOnResponse {}

impl<B> OnResponse<B> for DefaultOnResponse {
    fn on_response(self, response: &Response<B>, duration: Duration, span: &Span) {
        let latency = Latency::new(duration);
        tracing::debug!(parent: span, latency = %latency, status = %response.status(), "finished processing request");
    }
}
