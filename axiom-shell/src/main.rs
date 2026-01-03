use anyhow::{Result, Context};
use std::sync::Arc;
use tokio::net::{UnixListener, TcpListener};
use tokio::io::AsyncReadExt;
use tracing::{info, error, warn};
use axum::{Router, routing::get, extract::{Path, State}, response::Html, Json};
use std::process::Command;

mod runtime;
mod bridge;
mod session;
mod adapters;
mod supervisor;
mod egress;
mod db;
mod resilience;

use crate::runtime::WasmSupervisor;

const SOCKET_PATH: &str = "/tmp/axiom_shell.sock";
const HTTP_PORT: &str = "0.0.0.0:9000";

#[derive(serde::Deserialize)]
struct DeployPayload {
    pub tomain_id: String,
    pub wasm_base64: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("🚀 Booting Axiom Shell Supervisor...");

    // Recursive Startup: Ensure CCP is running before starting Shell
    ensure_ccp_running().await;

    let supervisor = Arc::new(WasmSupervisor::new().await?);
    
    // Load bindings from ~/.axiom/session.json into the live egress DashMap
    supervisor.egress.reload_from_registry();
    let _ = supervisor.db_registry.reload_from_registry().await;
    let _ = supervisor.resilience.reload_from_registry().await;
    
    // Cleanup port 9000 if in use
    cleanup_port(9000);

    // 1. Health Monitoring Loop (Team-Aware Refactoring Section #5)
    let sv_clone = supervisor.clone();
    tokio::spawn(async move {
        info!("🩺 Starting background health monitoring loop (30s interval)...");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            if let Err(e) = sv_clone.clone().check_all_health().await {
                error!("Health monitor encountered error: {}", e);
            }
        }
    });

    // 2. Start HTTP Server for UI/Reflection (Team-Aware Refactoring Section #4)
    let supervisor_http = supervisor.clone();
    tokio::spawn(async move {
        let app = Router::new()
            .route("/", get(|| async { Html("<h1>Axiom Shell Status: ONLINE</h1>") }))
            // Reflection Route
            .route("/reflect/{tomain}", get(
                |Path(tomain): Path<String>, State(sv): State<Arc<WasmSupervisor>>| async move {
                    match sv.reflect(&tomain).await {
                        Ok(json) => {
                            // Rewrite the servers URL to include the tomain prefix
                            // so Swagger UI calls /{tomain}/{func} correctly
                            let patched = if let Ok(mut spec) = serde_json::from_str::<serde_json::Value>(&json) {
                                spec["servers"] = serde_json::json!([
                                    { "url": format!("http://localhost:9000/{}", tomain), "description": "Local Axiom Shell" }
                                ]);
                                spec.to_string()
                            } else {
                                json
                            };
                            axum::response::Response::builder()
                                .header("Content-Type", "application/json")
                                .header("Access-Control-Allow-Origin", "*")
                                .body(axum::body::Body::from(patched))
                                .unwrap()
                        },
                        Err(e) => axum::response::Response::builder()
                            .status(404)
                            .header("Access-Control-Allow-Origin", "*")
                            .body(axum::body::Body::from(format!("Error fetching manifest for {}: {}", tomain, e)))
                            .unwrap(),
                    }
                }
            ))
            // Invocation Route (Generic) - supports GET, POST, PUT, DELETE and CORS preflight
            .route("/{tomain}/{func}", axum::routing::any(
                |method: axum::http::Method,
                 Path((tomain, func)): Path<(String, String)>, 
                 uri: axum::http::Uri,
                 headers: axum::http::HeaderMap,
                 State(sv): State<Arc<WasmSupervisor>>,
                 body: axum::body::Bytes| async move {
                    // 1. Handle CORS preflight
                    if method == axum::http::Method::OPTIONS {
                        return axum::response::Response::builder()
                            .header("Access-Control-Allow-Origin", "*")
                            .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
                            .header("Access-Control-Allow-Headers", "*")
                            .body(axum::body::Body::empty())
                            .unwrap();
                    }

                    // 2. Upstream Resilience Guards
                    // a. Rate Limiting (Default 100 req/sec if not specified)
                    if !sv.resilience.traffic.check_upstream(&tomain, 100.0) {
                        return axum::response::Response::builder()
                            .status(axum::http::StatusCode::TOO_MANY_REQUESTS)
                            .header("Access-Control-Allow-Origin", "*")
                            .body(axum::body::Body::from("Rate Limit Exceeded (Upstream)"))
                            .unwrap();
                    }

                    // b. JWT Identity Validation (Pillar #9)
                    if sv.resilience.security.public_keys.contains_key(&tomain) {
                        let auth_valid = if let Some(auth_val) = headers.get("Authorization") {
                            if let Ok(auth_str) = auth_val.to_str() {
                                if auth_str.starts_with("Bearer ") {
                                    let token = &auth_str[7..];
                                    sv.resilience.security.validate_jwt(&tomain, token).is_ok()
                                } else { false }
                            } else { false }
                        } else { false };

                        if !auth_valid {
                            return axum::response::Response::builder()
                                .status(axum::http::StatusCode::UNAUTHORIZED)
                                .header("Access-Control-Allow-Origin", "*")
                                .body(axum::body::Body::from("Invalid or Missing Authorization Token"))
                                .unwrap();
                        }
                    }
                    let query_json = if method == axum::http::Method::POST 
                        || method == axum::http::Method::PUT 
                    {
                        // For POST/PUT: prefer the request body
                        if !body.is_empty() {
                            String::from_utf8_lossy(&body).to_string()
                        } else if let Some(query) = uri.query() {
                            // Fallback to query params if body is empty
                            let params: std::collections::HashMap<String, String> = 
                                url::form_urlencoded::parse(query.as_bytes())
                                    .map(|(k, v)| (k.to_string(), v.to_string()))
                                    .collect();
                            serde_json::to_string(&params).unwrap_or_else(|_| "{}".to_string())
                        } else {
                            "{}".to_string()
                        }
                    } else {
                        // For GET: use query params
                        if let Some(query) = uri.query() {
                            let params: std::collections::HashMap<String, String> = 
                                url::form_urlencoded::parse(query.as_bytes())
                                    .map(|(k, v)| (k.to_string(), v.to_string()))
                                    .collect();
                            serde_json::to_string(&params).unwrap_or_else(|_| "{}".to_string())
                        } else {
                            "{}".to_string()
                        }
                    };

                    match sv.call(&tomain, &func, query_json).await {
                        Ok(res) => axum::response::Response::builder()
                            .header("Content-Type", "text/plain")
                            .header("Access-Control-Allow-Origin", "*")
                            .body(axum::body::Body::from(res))
                            .unwrap(),
                        Err(e) => axum::response::Response::builder()
                            .status(500)
                            .header("Access-Control-Allow-Origin", "*")
                            .body(axum::body::Body::from(format!("Invocation Error: {}", e)))
                            .unwrap(),
                    }
                }
            ))
            // Hot-reload endpoint: CCP calls this after any binding change
            .route("/admin/reload-bindings", axum::routing::post(
                |State(sv): State<Arc<WasmSupervisor>>| async move {
                    sv.egress.reload_from_registry();
                    let _ = sv.db_registry.reload_from_registry().await;
                    let _ = sv.resilience.reload_from_registry().await;
                    axum::response::Response::builder()
                        .header("Content-Type", "text/plain")
                        .body(axum::body::Body::from("Bindings reloaded"))
                        .unwrap()
                }
            ))
            // Perspective Switcher: CCP calls this to change context (GREEN/BLUE/RED)
            .route("/admin/perspective", axum::routing::post(
                |State(sv): State<Arc<WasmSupervisor>>, Json(payload): Json<serde_json::Value>| async move {
                    let id = payload["tomain_id"].as_str().unwrap_or_default();
                    let target = payload["target"].as_str().unwrap_or("GREEN");
                    match sv.update_perspective(id, target).await {
                        Ok(_) => axum::response::Response::builder()
                            .header("Content-Type", "text/plain")
                            .body(axum::body::Body::from(format!("Perspective switched to {}", target)))
                            .unwrap(),
                        Err(e) => axum::response::Response::builder()
                            .status(500)
                            .body(axum::body::Body::from(format!("Failed to switch perspective: {}", e)))
                            .unwrap(),
                    }
                }
            ))
            // Service Retirement: Flush memory slots
            .route("/admin/retire", axum::routing::post(
                |State(sv): State<Arc<WasmSupervisor>>, Json(payload): Json<serde_json::Value>| async move {
                    let id = payload["tomain_id"].as_str().unwrap_or_default();
                    let env = payload["env"].as_str().unwrap_or("GREEN");
                    let _ = sv.retire_service(id, env).await;
                    axum::response::Response::builder()
                        .header("Content-Type", "text/plain")
                        .body(axum::body::Body::from(format!("Retired {} from {} slot", id, env)))
                        .unwrap()
                }
            ))
            // Admin health check per tenant/env
            .route("/admin/health/{id}/{env}", get(
                |Path((id, env)): Path<(String, String)>, State(sv): State<Arc<WasmSupervisor>>| async move {
                    let tenants = sv.manager.tenants.read().await;
                    if let Some(env_map) = tenants.get(&id) {
                        if let Some(tenant) = env_map.get(&env.to_uppercase()) {
                             match crate::bridge::invoke_health(sv.clone(), tenant.clone()).await {
                                Ok(s) => return axum::response::Response::builder()
                                    .header("Content-Type", "text/plain")
                                    .header("Access-Control-Allow-Origin", "*")
                                    .body(axum::body::Body::from(s))
                                    .unwrap(),
                                Err(e) => return axum::response::Response::builder()
                                    .status(500)
                                    .header("Access-Control-Allow-Origin", "*")
                                    .body(axum::body::Body::from(format!("Unhealthy: {}", e)))
                                    .unwrap(),
                            }
                        }
                    }
                    axum::response::Response::builder()
                        .status(404)
                        .header("Access-Control-Allow-Origin", "*")
                        .body(axum::body::Body::from("Tenant/Env not found"))
                        .unwrap()
                }
            ))
            .route("/admin/tenants", get(
                |State(sv): State<Arc<WasmSupervisor>>| async move {
                    let tenants = sv.manager.tenants.read().await;
                    let ids: Vec<String> = tenants.keys().cloned().collect();
                    axum::response::Response::builder()
                        .header("Content-Type", "application/json")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(axum::body::Body::from(serde_json::to_string(&ids).unwrap()))
                        .unwrap()
                }
            ))
            .with_state(supervisor_http);
            
        let tcp_listener = TcpListener::bind(HTTP_PORT).await.expect("Failed to bind Shell HTTP port");
        info!("🌐 Shell HTTP Server active on http://localhost:9000");
        if let Err(e) = axum::serve(tcp_listener, app).await {
            error!("HTTP Server crashed: {:#}", e);
        }
    });

    // 3. Start Hot-Swap Unix Socket Listener
    let _ = std::fs::remove_file(SOCKET_PATH);
    let listener = UnixListener::bind(SOCKET_PATH).context("Failed to bind Unix socket")?;
    info!("🎧 Hot-swap socket listening on {}", SOCKET_PATH);

    // 4. Session Watcher
    let sv_session = supervisor.clone();
    tokio::spawn(async move {
        if let Err(e) = session::watch_sessions(sv_session).await {
            error!("Session watcher failed: {}", e);
        }
    });

    loop {
        match listener.accept().await {
            Ok((mut socket, _)) => {
                let sv = supervisor.clone();
                tokio::spawn(async move {
                    let mut buffer = Vec::new();
                    if let Ok(_) = socket.read_to_end(&mut buffer).await {
                        if let Ok(payload) = serde_json::from_slice::<DeployPayload>(&buffer) {
                            // Local dev deployment always targets GREEN
                            let _ = sv.deploy_kernel(&payload.tomain_id, "GREEN".to_string(), payload.wasm_base64).await;
                            // Also set initial perspective to GREEN
                            let _ = sv.update_perspective(&payload.tomain_id, "GREEN").await;
                        }
                    }
                });
            }
            Err(e) => error!("Socket error: {}", e),
        }
    }
}

fn cleanup_port(port: u16) {
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("lsof -ti:{} | xargs kill -9", port))
        .status();
}

async fn ensure_ccp_running() {
    let client = reqwest::Client::new();
    // Check if CCP backend is responsive
    let res = client.get("http://localhost:3000/api/v1/tomains").send().await;
    
    if res.is_err() {
        warn!("📡 Axiom CCP Central Control Plane not detected on port 3000.");
        info!("🚀 Attempting to start CCP automatically...");
        
        let ccp_dir = if std::path::Path::new("axiom-ccp").exists() {
            "axiom-ccp"
        } else if std::path::Path::new("../axiom-ccp").exists() {
            "../axiom-ccp"
        } else if std::path::Path::new("../../axiom-ccp").exists() {
            "../../axiom-ccp"
        } else {
            "../axiom-ccp"
        };
        
        let cmd = format!("cd {} && nohup ./dev.sh > /dev/null 2>&1 &", ccp_dir);
        let _ = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .spawn();
            
        info!("✅ CCP startup sequence triggered via dev.sh (Backend + Frontend)");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    } else {
        info!("📡 Axiom CCP is already healthy on port 3000.");
    }
}
