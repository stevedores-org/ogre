use axum::{routing::get, Json, Router};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let app = Router::new()
        .route("/health", get(health))
        .route("/readiness", get(readiness))
        .route("/version", get(version));

    let addr = std::env::var("OGRE_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind ogre-svc listener");
    tracing::info!(%addr, "ogre-svc listening");
    axum::serve(listener, app).await.expect("serve ogre-svc");
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "service": "ogre-svc" }))
}

async fn readiness() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ready", "service": "ogre-svc" }))
}

async fn version() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "ogre-svc",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
