mod error;
mod extractor;
mod users;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use anyhow::Context;
use axum::{routing::get, Router};
pub use error::{Error, ResultExt};

pub type Result<T, E = Error> = std::result::Result<T, E>;

use crate::config::Config;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub(crate) struct ApiContext {
    config: Arc<Config>,
    db: PgPool,
}

pub async fn serve(config: Config, db: PgPool) -> anyhow::Result<()> {
    let api_context = ApiContext {
        config: Arc::new(config),
        db,
    };

    let app = api_router(api_context);

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3000);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .await
        .context("error running http server")
}

fn api_router(api_context: ApiContext) -> Router {
    Router::new()
        .route("/hello", get(|| async { "Hello" }))
        .merge(users::router())
        .layer(TraceLayer::new_for_http())
        .with_state(api_context)
}
