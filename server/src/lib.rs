mod latency;
mod middleware;

use std::{
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};

use axum::{
    Extension,
    Router,
    Server,
};
use middleware::{
    DefaultMakeSpan,
    DefaultOnRequest,
    DefaultOnResponse,
};
use tower::ServiceBuilder;
use tower_http::{
    request_id::MakeRequestUuid,
    timeout::TimeoutLayer,
    trace::TraceLayer,
    ServiceBuilderExt,
};

pub async fn start(router: Router, port: u16) {
    let server = Server::bind(&SocketAddr::from(([127, 0, 0, 1], port))).serve(
        create_router(router)
            .layer(Extension(Arc::new(1)))
            .into_make_service(),
    );

    tracing::info!("listening on {}", server.local_addr());

    server.await.unwrap();
}

fn create_router(router: Router) -> Router {
    let middleware = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default())
                .on_request(DefaultOnRequest::default())
                .on_response(DefaultOnResponse::default()),
        )
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .propagate_x_request_id()
        .compression();

    router.layer(middleware)
}
