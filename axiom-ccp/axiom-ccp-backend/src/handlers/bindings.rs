use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::{info, instrument};
use crate::handlers::registry::AppState;

const SHELL_BASE_URL: &str = "http://localhost:9000";

#[derive(Debug, Deserialize)]
pub struct RegisterBindingRequest {
    pub tomain_id: String,
    pub alias: String,
    pub physical_url: String,
    #[allow(dead_code)]
    pub environment: String,
}

#[derive(Debug, Deserialize)]
pub struct ResolveBindingQuery {
    pub tomain_id: String,
    pub alias: String,
    #[allow(dead_code)]
    pub environment: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteBindingRequest {
    pub tomain_id: String,
    pub alias: String,
}

/// POST /api/v1/bindings
/// Register or update a binding, flush to registry.json, then push hot-reload to Shell.
#[instrument(skip(state))]
pub async fn register_binding(
    State(state): State<AppState>,
    Json(payload): Json<RegisterBindingRequest>,
) -> impl IntoResponse {
    {
        let mut reg = state.registry.write().await;
        reg.bindings
            .entry(payload.tomain_id.clone())
            .or_default()
            .entry(payload.environment.to_uppercase())
            .or_default()
            .insert(payload.alias.clone(), payload.physical_url.clone());
        reg.flush();
        info!("✅ Binding registered: {} ({}) → {} = {}", payload.tomain_id, payload.environment, payload.alias, payload.physical_url);
    }

    // Push hot-reload to Shell (fire-and-forget — don't block the response)
    tokio::spawn(push_reload_to_shell());

    (StatusCode::OK, "Binding registered successfully")
}

/// DELETE /api/v1/bindings (via POST with JSON body for simplicity)
#[instrument(skip(state))]
pub async fn delete_binding(
    State(state): State<AppState>,
    Json(payload): Json<DeleteBindingRequest>,
) -> impl IntoResponse {
    {
        let mut reg = state.registry.write().await;
        if let Some(tomain_bindings) = reg.bindings.get_mut(&payload.tomain_id) {
            tomain_bindings.remove(&payload.alias);
        }
        reg.flush();
    }

    tokio::spawn(push_reload_to_shell());
    (StatusCode::OK, "Binding deleted")
}

/// GET /api/v1/bindings/resolve?tomain_id=...&alias=...&environment=...
#[instrument(skip(state))]
pub async fn resolve_binding(
    State(state): State<AppState>,
    Query(query): Query<ResolveBindingQuery>,
) -> impl IntoResponse {
    let reg = state.registry.read().await;
    let env = query.environment.to_uppercase();
    match reg.bindings.get(&query.tomain_id)
        .and_then(|e| e.get(&env))
        .and_then(|m| m.get(&query.alias)) {
        Some(url) => (StatusCode::OK, url.clone()).into_response(),
        None => (StatusCode::NOT_FOUND, format!("No binding '{}' in '{}' for '{}'", query.alias, env, query.tomain_id)).into_response(),
    }
}

/// GET /api/v1/bindings
#[instrument(skip(state))]
pub async fn list_bindings(State(state): State<AppState>) -> impl IntoResponse {
    let reg = state.registry.read().await;
    let mut bindings = Vec::new();
    for (tomain_id, env_map) in &reg.bindings {
        for (env, alias_map) in env_map {
            for (alias, physical_url) in alias_map {
                bindings.push(serde_json::json!({
                    "id": format!("{}:{}:{}", tomain_id, env, alias),
                    "tomain_id": tomain_id,
                    "alias": alias,
                    "physical_url": physical_url,
                    "environment": env,
                }));
            }
        }
    }
    (StatusCode::OK, Json(bindings))
}

/// POST to Shell's /admin/reload-bindings — tells it to re-read registry.json
pub async fn push_reload_to_shell() {
    let client = reqwest::Client::new();
    match client.post(format!("{}/admin/reload-bindings", SHELL_BASE_URL))
        .send().await 
    {
        Ok(r) if r.status().is_success() => info!("🔄 Shell hot-reload triggered successfully"),
        Ok(r) => info!("⚠️ Shell hot-reload returned {}", r.status()),
        Err(e) => info!("⚠️ Shell not reachable for hot-reload (OK if Shell is down): {}", e),
    }
}
