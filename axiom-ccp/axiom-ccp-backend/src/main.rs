mod handlers;

use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use handlers::registry::{AxiomRegistry, AppState};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "axiom_ccp_backend=info,tower_http=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load registry from ~/.axiom/registry.json (creates if missing)
    let registry = AxiomRegistry::load_or_create();
    info!("✅ Loaded Axiom Registry ({} tomains, {} binding sets)",
        registry.tomains.len(),
        registry.bindings.len()
    );

    let app_state = AppState {
        registry: Arc::new(RwLock::new(registry)),
    };

    let app = Router::new()
        .route("/api/v1/tomains", get(handlers::tomain::list_tomains).post(handlers::tomain::register_tomain))
        .route("/api/v1/tomains/{id}", get(handlers::tomain::get_tomain).delete(handlers::tomain::delete_tomain))
        .route("/api/v1/tomains/{id}/manifest", get(handlers::tomain::get_manifest).post(handlers::tomain::update_manifest))
        .route("/api/v1/tomains/{id}/promote", post(handlers::tomain::promote_tomain))
        .route("/api/v1/tomains/{id}/promote/feature", post(handlers::tomain::promote_feature))
        .route("/api/v1/tomains/{id}/features", post(handlers::tomain::register_feature))
        .route("/api/v1/tomains/{id}/features/{feature_name}/wasm", post(handlers::tomain::upload_feature_wasm))
        .route("/api/v1/tomains/{id}/retire", post(handlers::tomain::retire_tomain))
        .route("/api/v1/tomains/resolve/{*tomain}", get(handlers::tomain::resolve_tomain))
        .route("/api/v1/bindings", get(handlers::bindings::list_bindings).post(handlers::bindings::register_binding))
        .route("/api/v1/bindings/resolve", get(handlers::bindings::resolve_binding))
        .route("/api/v1/bindings/delete", post(handlers::bindings::delete_binding))
        .route("/api/v1/docs/{package_id}", get(handlers::docs::get_swagger_ui))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("🚀 Axiom CCP Backend listening on {} (DB-Free mode)", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
