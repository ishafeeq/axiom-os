/// Egress Guard — resolves alias → physical URL using the live in-memory DashMap.
/// Updated by CCP via POST /admin/reload-bindings without any restart.
use anyhow::{Result, anyhow};
use dashmap::DashMap;
use std::sync::Arc;
use serde_json::Value;
use tracing::{info, warn};

pub struct EgressResolver {
    /// (tomain_id, env, alias) → physical_url — hot-updated by CCP
    pub bindings: Arc<DashMap<(String, String, String), String>>,
    /// tomain_id → { logical_name → alias_name (@main-db) }
    pub manifests: Arc<DashMap<(String, String), String>>,
}

impl EgressResolver {
    pub fn new() -> Self {
        Self {
            bindings: Arc::new(DashMap::new()),
            manifests: Arc::new(DashMap::new()),
        }
    }

    /// Load all bindings from ~/.axiom/registry.json into the live DashMap.
    pub fn reload_from_registry(&self) {
        let path = dirs::home_dir()
            .unwrap_or_default()
            .join(".axiom")
            .join("session.json");

        match std::fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<Value>(&content) {
                    Ok(json) => {
                        self.bindings.clear();
                        if let Some(all_bindings) = json.get("bindings").and_then(|b| b.as_object()) {
                            for (tomain_id, env_map) in all_bindings {
                                if let Some(envs) = env_map.as_object() {
                                    for (env, aliases) in envs {
                                        if let Some(alias_map) = aliases.as_object() {
                                            for (alias, url) in alias_map {
                                                if let Some(url_str) = url.as_str() {
                                                    self.bindings.insert(
                                                        (tomain_id.clone(), env.clone(), alias.clone()),
                                                        url_str.to_string(),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(all_manifests) = json.get("manifests").and_then(|m| m.as_object()) {
                            self.manifests.clear();
                            for (tomain_id, logical_map) in all_manifests {
                                if let Some(map) = logical_map.as_object() {
                                    for (logical_name, alias) in map {
                                        if let Some(alias_str) = alias.as_str() {
                                            self.manifests.insert(
                                                (tomain_id.clone(), logical_name.clone()),
                                                alias_str.to_string(),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        info!("🔄 Egress: Reloaded {} bindings and {} manifests from session registry", self.bindings.len(), self.manifests.len());
                    }
                    Err(e) => warn!("Failed to parse session.json: {}", e),
                }
            }
            Err(_) => info!("No session.json found — starting with empty bindings"),
        }
    }

    /// Resolve alias → physical URL. Handles 2-step logical resolution.
    pub async fn resolve(&self, tomain_id: &str, logical_name: &str, environment: &str) -> Result<String> {
        // 1. Check if it's a logical name mapped in axiom.toml
        let alias = if let Some(a) = self.manifests.get(&(tomain_id.to_string(), logical_name.to_string())) {
            a.value().clone()
        } else {
            logical_name.to_string()
        };

        // 2. Resolve the alias (e.g. @main-db) to a physical URL
        let key = (tomain_id.to_string(), environment.to_string(), alias.clone());
        match self.bindings.get(&key) {
            Some(url) => {
                info!("✅ Egress: Resolved '{}' ('{}') → '{}' in {} context", logical_name, alias, url.value(), environment);
                Ok(url.clone())
            }
            None => {
                // Fallback: Check if there's an environment-independent binding
                let global_key = (tomain_id.to_string(), "GLOBAL".to_string(), alias.clone());
                if let Some(url) = self.bindings.get(&global_key) {
                     return Ok(url.clone());
                }
                
                warn!("🛑 Egress: No binding found for alias '{}' (tomain: {})", alias, tomain_id);
                Err(anyhow!("No binding for alias '{}'", alias))
            }
        }
    }
}
