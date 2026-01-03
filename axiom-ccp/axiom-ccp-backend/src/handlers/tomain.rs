use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use chrono::Utc;
use crate::handlers::registry::{AppState, TomainEntry};

fn compute_commits_ahead(repo_url: &Option<String>, features: &mut std::collections::HashMap<String, crate::handlers::registry::FeatureDetail>) {
    if let Some(repo_path) = repo_url {
        for (feat_name, detail) in features.iter_mut() {
            let branch_name = detail.branch.as_deref().unwrap_or("main");
            if branch_name.starts_with("feature/") {
                if let Ok(output) = std::process::Command::new("git")
                    .args(["--git-dir", repo_path, "rev-list", "--count", &format!("main..{}", branch_name)])
                    .output() {
                    if output.status.success() {
                        if let Ok(count_str) = String::from_utf8(output.stdout) {
                            detail.commits_ahead = count_str.trim().parse().ok();
                        }
                    }
                }
            } else {
                 if let Ok(output) = std::process::Command::new("git")
                    .args(["--git-dir", repo_path, "rev-list", "--count", &format!("main..feature/{}", feat_name)])
                    .output() {
                    if output.status.success() {
                        if let Ok(count_str) = String::from_utf8(output.stdout) {
                            detail.commits_ahead = count_str.trim().parse().ok();
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ResolveQuery {
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterTomainRequest {
    pub name: String,
    pub owner: String,
    pub team_name: Option<String>,
    pub package_name: Option<String>,
    pub creator_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionMetadata {
    pub environment: String,
    pub database_url: String,
    pub cache_url: String,
    pub message_queue: String,
}

/// GET /api/v1/tomains
#[instrument(skip(state))]
pub async fn list_tomains(State(state): State<AppState>) -> impl IntoResponse {
    let reg = state.registry.read().await;
    
    // Attempt to fetch active tenants from Shell
    let active_tenants: Vec<String> = match reqwest::get("http://localhost:9000/admin/tenants").await {
        Ok(res) => res.json::<Vec<String>>().await.unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    let tomains: Vec<serde_json::Value> = reg.tomains.iter().map(|(id, entry)| {
        let rate_limit = reg.rate_limits.as_ref()
            .and_then(|rl| rl.get(id))
            .cloned()
            .unwrap_or(serde_json::Value::Null);
            
        let has_public_key = reg.public_keys.as_ref()
            .map(|pk| pk.contains_key(id))
            .unwrap_or(false);

        // Sync health status with Shell
        let health_status = if active_tenants.contains(id) {
            entry.status.clone()
        } else {
            "Inactive".to_string()
        };

        let mut features = entry.features.clone();
        compute_commits_ahead(&entry.repo_url, &mut features);

        serde_json::json!({
            "id": id,
            "name": id,
            "owner": entry.owner,
            "health_status": health_status,
            "package_name": entry.package_name,
            "creator_name": entry.creator_name,
            "team_name": entry.team_name,
            "created_at": entry.created_at,
            "perspective": entry.perspective,
            "min_perspective": entry.min_perspective,
            "wasm_hashes": entry.wasm_hashes,
            "rate_limit": rate_limit,
            "has_public_key": has_public_key,
            "api_count": entry.apis.as_ref().map(|a| a.len()).unwrap_or(0),
            "apis": entry.apis,
            "repo_url": entry.repo_url,
            "features": features,
        })
    }).collect();
    (StatusCode::OK, Json(tomains))
}

/// POST /api/v1/tomains
#[instrument(skip(state))]
pub async fn register_tomain(
    State(state): State<AppState>,
    Json(payload): Json<RegisterTomainRequest>,
) -> impl IntoResponse {
    let mut reg = state.registry.write().await;

    if reg.tomains.contains_key(&payload.name) {
        return (StatusCode::CONFLICT, Json(serde_json::json!({"error": "Tomain already exists"}))).into_response();
    }

    let now = Utc::now().to_rfc3339();
    let entry = TomainEntry {
        owner: payload.owner.clone(),
        status: "Active".to_string(),
        package_name: payload.package_name.clone(),
        creator_name: payload.creator_name.clone(),
        team_name: payload.team_name.clone(),
        created_at: now,
        perspective: "DEV".to_string(),
        min_perspective: "DEV".to_string(),
        wasm_hashes: std::collections::HashMap::new(),
        repo_url: None,
        features: std::collections::HashMap::new(),
        wit: None,
        apis: None,
    };

    reg.tomains.insert(payload.name.clone(), entry);
    reg.flush();

    (StatusCode::CREATED, Json(serde_json::json!({
        "message": format!("Tomain '{}' registered.", payload.name),
        "name": payload.name
    }))).into_response()
}
pub async fn delete_tomain(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut reg = state.registry.write().await;
    reg.delete_tomain(&id);
    (StatusCode::OK, "Tomain deleted")
}

pub async fn get_tomain(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let reg = state.registry.read().await;
    
    match reg.tomains.get(&id) {
        Some(entry) => {
             // Sync health status with Shell (mocked logic similar to list)
             let active_tenants: Vec<String> = match reqwest::get("http://localhost:9000/admin/tenants").await {
                Ok(res) => res.json::<Vec<String>>().await.unwrap_or_default(),
                Err(_) => Vec::new(),
             };
             
             let health_status = if active_tenants.contains(&id) {
                entry.status.clone()
             } else {
                "Inactive".to_string()
             };

             let has_public_key = reg.public_keys.as_ref()
                .map(|pk| pk.contains_key(&id))
                .unwrap_or(false);

             let rate_limit = reg.rate_limits.as_ref()
                .and_then(|rl| rl.get(&id))
                .cloned()
                .unwrap_or(serde_json::Value::Null);

             let mut features = entry.features.clone();
             compute_commits_ahead(&entry.repo_url, &mut features);

             Json(serde_json::json!({
                "id": id,
                "name": id,
                "owner": entry.owner,
                "health_status": health_status,
                "package_name": entry.package_name,
                "creator_name": entry.creator_name,
                "team_name": entry.team_name,
                "created_at": entry.created_at,
                "perspective": entry.perspective,
                "min_perspective": entry.min_perspective,
                "wasm_hashes": entry.wasm_hashes,
                "rate_limit": rate_limit,
                "has_public_key": has_public_key,
                "api_count": entry.apis.as_ref().map(|a| a.len()).unwrap_or(0),
                "apis": entry.apis.clone(),
                "repo_url": entry.repo_url.clone(),
                "features": features,
             })).into_response()
        },
        None => (StatusCode::NOT_FOUND, "Tomain not found").into_response(),
    }
}

/// GET /api/v1/tomains/{*tomain}/resolve?color=GREEN
#[instrument(skip(state))]
pub async fn resolve_tomain(
    State(state): State<AppState>,
    Path(tomain): Path<String>,
    Query(query): Query<ResolveQuery>,
) -> impl IntoResponse {
    let color = query.color.unwrap_or_else(|| "DEV".to_string()).to_uppercase();
    let reg = state.registry.read().await;

    if !reg.tomains.contains_key(&tomain) {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tomain not found"}))).into_response();
    }

    // Attempt to pull from 'infra' or 'secrets' in the registry
    let database_url = reg.secrets.get(&tomain)
        .and_then(|s| s.get("DATABASE_URL"))
        .cloned()
        .unwrap_or_else(|| {
            match color.as_str() {
                "DEV" => format!("postgres://dev-db.internal/{}", tomain),
                "QA" => format!("postgres://qa-db.internal/{}", tomain),
                "STAGING" => format!("postgres://staging-db.internal/{}", tomain),
                "PROD" => format!("postgres://prod-db.internal/{}", tomain),
                _ => format!("postgres://localhost:5432/{}", tomain),
            }
        });

    let metadata = ConnectionMetadata {
        environment: format!("Context ({})", color),
        database_url,
        cache_url: reg.infra.get("cache_url").cloned().unwrap_or_else(|| "redis://localhost:6379".to_string()),
        message_queue: reg.infra.get("message_queue").cloned().unwrap_or_else(|| "nats://localhost:4222".to_string()),
    };

    (StatusCode::OK, Json(metadata)).into_response()
}

/// GET /api/v1/tomains/{id}/manifest
#[instrument(skip(state))]
pub async fn get_manifest(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let reg = state.registry.read().await;
    
    match reg.tomains.get(&id) {
        Some(entry) => {
            let manifest = serde_json::json!({
                "tomain_id": id,
                "wit": entry.wit,
                "perspective": entry.perspective,
                "capabilities": ["http", "persistence", "tracing"],
                "repo_url": entry.repo_url,
                "features": entry.features,
            });
            (StatusCode::OK, Json(manifest)).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Tomain not found").into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct PromoteRequest {
    pub target: String,
    pub wasm_base64: Option<String>,
}

/// POST /api/v1/tomains/{id}/promote
#[instrument(skip(state))]
pub async fn promote_tomain(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<PromoteRequest>,
) -> impl IntoResponse {
    let mut reg = state.registry.write().await;
    
    match reg.tomains.get_mut(&id) {
        Some(entry) => {
            let target = payload.target.to_uppercase();
            
            // Pillar #8: Safety Gate
            if target == "PROD" {
                let health_res = reqwest::get(format!("http://localhost:9000/admin/health/{}/STAGING", id)).await;
                let is_healthy = match health_res {
                    Ok(res) => res.status().is_success(),
                    Err(_) => false,
                };
                if !is_healthy {
                    return (StatusCode::PRECONDITION_FAILED, "Promotion Blocked: Service must be Healthy in BLUE before RED promotion").into_response();
                }
            }

            entry.perspective = target.clone();
            if let Some(wasm) = payload.wasm_base64 {
                entry.wasm_hashes.insert(target.clone(), wasm);
            }
            reg.flush();
            (StatusCode::OK, "Promotion successful").into_response()
        }
        None => (StatusCode::NOT_FOUND, "Tomain not found").into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct FeaturePromoteRequest {
    pub feature_name: String,
    pub from: String,
    pub to: String,
}

/// POST /api/v1/tomains/{id}/promote/feature
#[instrument(skip(state))]
pub async fn promote_feature(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<FeaturePromoteRequest>,
) -> impl IntoResponse {
    let mut reg = state.registry.write().await;
    let from = payload.from.to_uppercase();
    let to = payload.to.to_uppercase();

    match reg.tomains.get_mut(&id) {
        Some(entry) => {
            let mut feature_wasm = None;
            if let Some(feature) = entry.features.get(&payload.feature_name) {
                feature_wasm = feature.wasm_hash.clone();
            }

            let wasm_to_promote = feature_wasm.or_else(|| entry.wasm_hashes.get(&from).cloned());

            if let Some(wasm) = wasm_to_promote {
                entry.wasm_hashes.insert(to.clone(), wasm);
                entry.perspective = to.clone();
                
                // Track feature-to-environment mapping
                if let Some(feature) = entry.features.get_mut(&payload.feature_name) {
                    if !feature.environments.contains(&to) {
                        feature.environments.push(to.clone());
                    }
                }

                reg.flush();
                (StatusCode::OK, format!("Feature '{}' promoted from {} to {}", payload.feature_name, from, to)).into_response()
            } else {
                (StatusCode::BAD_REQUEST, format!("No wasm found in {} perspective or feature payload", from)).into_response()
            }
        }
        None => (StatusCode::NOT_FOUND, "Tomain not found").into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct RetireRequest {
    pub env: String,
}

/// POST /api/v1/tomains/{id}/retire
#[instrument(skip(state))]
pub async fn retire_tomain(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<RetireRequest>,
) -> impl IntoResponse {
    let mut reg = state.registry.write().await;
    let env = payload.env.to_uppercase();

    match reg.tomains.get_mut(&id) {
        Some(entry) => {
            entry.wasm_hashes.remove(&env);
            reg.flush();
            (StatusCode::OK, format!("Service retired from {}", env)).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Tomain not found").into_response(),
    }
}


#[derive(Debug, Deserialize)]
pub struct RegisterFeatureRequest {
    pub name: String,
    pub branch: String,
}

/// POST /api/v1/tomains/{id}/features
pub async fn register_feature(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<RegisterFeatureRequest>,
) -> impl IntoResponse {
    let mut reg = state.registry.write().await;
    
    match reg.tomains.get_mut(&id) {
        Some(entry) => {
            let feature = crate::handlers::registry::FeatureDetail {
                wasm_hash: None,
                branch: Some(payload.branch),
                status: "Active".to_string(),
                environments: vec!["DEV".to_string()], // Initial feature is always in DEV
                commits_ahead: None,
            };
            entry.features.insert(payload.name.clone(), feature);
            reg.flush();
            (StatusCode::CREATED, format!("Feature '{}' registered for tomain '{}'", payload.name, id)).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Tomain not found").into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct UploadFeatureWasmRequest {
    pub wasm_base64: String,
}

/// POST /api/v1/tomains/{id}/features/{feature_name}/wasm
pub async fn upload_feature_wasm(
    State(state): State<AppState>,
    Path((id, feature_name)): Path<(String, String)>,
    Json(payload): Json<UploadFeatureWasmRequest>,
) -> impl IntoResponse {
    let mut reg = state.registry.write().await;
    
    match reg.tomains.get_mut(&id) {
        Some(entry) => {
            if let Some(feature) = entry.features.get_mut(&feature_name) {
                feature.wasm_hash = Some(payload.wasm_base64);
                reg.flush();
                (StatusCode::OK, format!("Wasm binary uploaded for feature '{}'", feature_name)).into_response()
            } else {
                (StatusCode::NOT_FOUND, format!("Feature '{}' not found", feature_name)).into_response()
            }
        }
        None => (StatusCode::NOT_FOUND, "Tomain not found").into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateManifestRequest {
    pub resources: std::collections::HashMap<String, ResourceDef>,
    pub apis: Option<Vec<crate::handlers::registry::ApiDetail>>,
    pub vault_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceDef {
    pub alias: String,
    #[serde(rename = "type")]
    pub resource_type: String,
}

/// POST /api/v1/tomains/{id}/manifest
pub async fn update_manifest(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateManifestRequest>,
) -> impl IntoResponse {
    let mut reg = state.registry.write().await;
    
    let mut manifest_map = std::collections::HashMap::new();
    for (name, res) in payload.resources {
        manifest_map.insert(name, res.alias);
    }
    
    reg.manifests.insert(id.clone(), manifest_map);
    
    if let Some(apis) = payload.apis {
        if let Some(entry) = reg.tomains.get_mut(&id) {
            entry.apis = Some(apis);
        }
    }

    if let Some(vault) = payload.vault_path {
        if let Some(entry) = reg.tomains.get_mut(&id) {
            entry.repo_url = Some(vault);
        }
    }
    
    reg.flush();
    
    // Trigger Shell reload
    tokio::spawn(crate::handlers::bindings::push_reload_to_shell());

    (StatusCode::OK, "Manifest updated").into_response()
}
