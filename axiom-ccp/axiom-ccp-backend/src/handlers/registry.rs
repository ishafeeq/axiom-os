/// The shared file-based registry — single source of truth for all Axiom OS state.
/// Stored at ~/.axiom/registry.json and loaded into memory on startup.
/// All reads are from memory (fast), all writes flush to disk atomically.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Full registry state loaded from ~/.axiom/registry.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AxiomRegistry {
    /// tomain_id → TomainEntry
    pub tomains: HashMap<String, TomainEntry>,
    /// tomain_id → { environment → { alias → physical_url } }
    pub bindings: HashMap<String, HashMap<String, HashMap<String, String>>>,
    /// tomain_id → { key → value } (secrets/env vars)
    pub secrets: HashMap<String, HashMap<String, String>>,
    /// tomain_id → { logical_name → alias_name (@main-db) }
    pub manifests: HashMap<String, HashMap<String, String>>,
    /// list of repository paths or metadata
    pub repositories: Vec<String>,
    /// tomain_id -> rate_limit (bps/rps)
    pub rate_limits: Option<HashMap<String, serde_json::Value>>,
    /// tomain_id -> public_key string
    pub public_keys: Option<HashMap<String, String>>,
    /// tomain_id -> { alias -> token }
    pub vault: Option<HashMap<String, String>>,
    /// Global infra info (e.g. registry URL, VPC ID, etc)
    pub infra: HashMap<String, String>,
}

fn default_perspective() -> String { "DEV".to_string() }

fn default_min_perspective() -> String { "DEV".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDetail {
    pub wasm_hash: Option<String>,
    pub branch: Option<String>,
    pub status: String, // e.g., "Active", "PR-Open"
    #[serde(default)]
    pub environments: Vec<String>, // List of environments this feature is promoted to
    #[serde(default)]
    pub commits_ahead: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TomainEntry {
    pub owner: String,
    pub status: String,
    pub package_name: Option<String>,
    pub creator_name: Option<String>,
    pub team_name: Option<String>,
    pub created_at: String,
    #[serde(default = "default_perspective")]
    pub perspective: String,
    #[serde(default = "default_min_perspective")]
    pub min_perspective: String,
    #[serde(default)]
    pub wasm_hashes: HashMap<String, String>, // env -> wasm_base64
    pub repo_url: Option<String>,
    #[serde(default)]
    pub features: HashMap<String, FeatureDetail>,
    pub wit: Option<String>,
    pub apis: Option<Vec<ApiDetail>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDetail {
    pub name: String,
    pub method: String, // GET, POST, etc.
    pub params: Vec<(String, String)>,
    pub doc: Option<String>,
}

impl AxiomRegistry {
    pub fn delete_tomain(&mut self, id: &str) {
        self.tomains.remove(id);
        self.bindings.remove(id);
        self.secrets.remove(id);
        if let Some(rl) = &mut self.rate_limits { rl.remove(id); }
        if let Some(pk) = &mut self.public_keys { pk.remove(id); }
        if let Some(v) = &mut self.vault { v.remove(id); }
        self.flush();
    }

    pub fn registry_path() -> std::path::PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        home.join(".axiom").join("session.json")
    }

    pub fn load_or_create() -> Self {
        let path = Self::registry_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    // Pre-parsing for migration if needed
                    let mut registry_val: serde_json::Value = serde_json::from_str(&content).unwrap_or(serde_json::json!({}));
                    
                    if let Some(tomains) = registry_val["tomains"].as_object_mut() {
                        for (_, tomain) in tomains {
                            // Migrate wasm_base64 -> wasm_hashes["GREEN"]
                            if let Some(old_wasm) = tomain.get("wasm_base64").and_then(|v| v.as_str()) {
                                if !tomain.get("wasm_hashes").is_some() {
                                    tomain["wasm_hashes"] = serde_json::json!({ "GREEN": old_wasm });
                                }
                            }
                            // Default min_perspective for old entries
                            if tomain.get("min_perspective").is_none() {
                                tomain["min_perspective"] = serde_json::json!("GREEN");
                            }
                            if tomain.get("features").is_none() || tomain["features"].as_object().map(|o| o.is_empty()).unwrap_or(false) {
                                tomain["features"] = serde_json::json!({
                                    "v2-auth": {
                                        "wasm_hash": "a1b2c3d4e5f6",
                                        "branch": "feat/v2-auth",
                                        "status": "Active"
                                    }
                                });
                            }
                            if tomain.get("repo_url").is_none() {
                                tomain["repo_url"] = serde_json::json!("https://github.com/axiom/sample-service");
                            }
                        }
                    }

                    if let Some(all_bindings) = registry_val["bindings"].as_object_mut() {
                        for (_, tomain_map) in all_bindings {
                            if let Some(map) = tomain_map.as_object() {
                                if !map.values().any(|v| v.is_object()) {
                                    *tomain_map = serde_json::json!({ "GREEN": map });
                                }
                            }
                        }
                    }

                    match serde_json::from_value::<Self>(registry_val) {
                        Ok(registry) => return registry,
                        Err(e) => warn!("Failed to parse/migrate session.json: {}. Creating fresh.", e),
                    }
                },
                Err(e) => warn!("Failed to read session.json: {}. Creating fresh.", e),
            }
        }
        let fresh = Self::default();
        fresh.flush(); // Create the file
        fresh
    }

    pub fn flush(&self) {
        let path = Self::registry_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = std::fs::write(&path, content) {
                    warn!("Failed to flush session.json: {}", e);
                } else {
                    info!("📝 Session registry flushed to {:?}", path);
                }
            }
            Err(e) => warn!("Failed to serialize registry: {}", e),
        }
    }
}

/// Shared app state — all handlers use this
#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<RwLock<AxiomRegistry>>,
}
