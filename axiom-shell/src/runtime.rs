use anyhow::{Result, Context};
use std::sync::Arc;
use tracing::info;
use crate::supervisor::TenantManager;
use crate::adapters::InfraRegistry;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub struct WasmSupervisor {
    pub manager: TenantManager,
    pub registry: Arc<InfraRegistry>,
    pub egress: Arc<crate::egress::EgressResolver>,
    pub http_client: reqwest::Client,
    pub db_registry: Arc<crate::db::DatabaseRegistry>,
    pub resilience: Arc<crate::resilience::ResilienceManager>,
    pub perspective: Arc<dashmap::DashMap<String, String>>, // tomain_id -> GREEN/BLUE/RED
    pub audit_log: Arc<dashmap::DashMap<String, Vec<String>>>, // tomain_id -> entries
}

impl WasmSupervisor {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            manager: TenantManager::new(),
            registry: Arc::new(InfraRegistry::new()),
            egress: Arc::new(crate::egress::EgressResolver::new()),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()?,
            db_registry: Arc::new(crate::db::DatabaseRegistry::new()),
            resilience: Arc::new(crate::resilience::ResilienceManager::new()),
            perspective: Arc::new(dashmap::DashMap::new()),
            audit_log: Arc::new(dashmap::DashMap::new()),
        })
    }

    pub async fn update_perspective(&self, tomain_id: &str, target: &str) -> Result<()> {
        let target_env = target.to_uppercase();
        info!("🔄 Perspective shift for {}: -> {}", tomain_id, target_env);
        
        // Hot-Swap Logic: Ensure instance exists in the target slot
        if self.manager.get_tenant(tomain_id, &target_env).await.is_none() {
            info!("🔍 Target slot {} empty for {}. Fetching from CCP...", target_env, tomain_id);
            // Fetch from CCP (Registry)
            let res = self.http_client.get("http://localhost:3000/api/v1/tomains").send().await?;
            let tomains: Vec<serde_json::Value> = res.json().await?;
            
            if let Some(tomain) = tomains.iter().find(|t| t["id"] == tomain_id) {
                if let Some(wasm_base64) = tomain["wasm_hashes"][&target_env].as_str() {
                    self.deploy_kernel(tomain_id, target_env.clone(), wasm_base64.to_string()).await?;
                    info!("✅ Hot-Swap complete: {} now active in {} slot", tomain_id, target_env);
                }
            }
        }

        self.perspective.insert(tomain_id.to_string(), target_env.clone());
        if target_env == "RED" {
            info!("🔴 AUDIT MODE ENABLED for tomain: {}", tomain_id);
            self.audit_log.entry(tomain_id.to_string()).or_insert_with(Vec::new);
        }
        Ok(())
    }

    pub async fn deploy_kernel(&self, tomain_id: &str, env: String, wasm_base64: String) -> Result<()> {
        let tenant_count = self.manager.tenants.read().await.len();
        if tenant_count >= 4 && !self.manager.tenants.read().await.contains_key(tomain_id) {
            return Err(anyhow::anyhow!("Shell capacity reached (max 4 active kernels). Please stop a service before deploying a new one."));
        }

        info!("Deploying kernel for Tomain: {} in {} slot", tomain_id, env);
        let wasm_bytes = BASE64.decode(wasm_base64).context("Failed to decode wasm base64")?;
        
        self.manager.register_tenant(tomain_id, &env, &wasm_bytes).await?;
        self.registry.update_status(tomain_id, "Active").await?;
        
        Ok(())
    }

    pub async fn retire_service(&self, tomain_id: &str, env: &str) -> Result<()> {
        self.manager.remove_tenant(tomain_id, env).await?;
        Ok(())
    }

    pub async fn reflect(self: Arc<Self>, tomain_id: &str) -> Result<String> {
        let env = self.get_perspective(tomain_id);
        let tenant = self.manager.get_tenant(tomain_id, &env).await
            .context(format!("Tenant '{}' not found in {} slot", tomain_id, env))?;
            
        crate::bridge::invoke_reflect(self.clone(), tenant).await
    }

    pub async fn call(self: Arc<Self>, tomain_id: &str, func_name: &str, query_json: String) -> Result<String> {
        let env = self.get_perspective(tomain_id);
        let tenant = self.manager.get_tenant(tomain_id, &env).await
            .context(format!("Tenant '{}' not found in {} slot", tomain_id, env))?;
            
        crate::bridge::invoke_call(self.clone(), tenant, func_name, query_json).await
    }

    pub fn get_perspective(&self, tomain_id: &str) -> String {
        self.perspective.get(tomain_id)
            .map(|v| v.value().clone())
            .unwrap_or_else(|| "GREEN".to_string())
    }

    pub async fn check_all_health(self: Arc<Self>) -> Result<()> {
        let tenants = self.manager.tenants.read().await;
        for (id, env_map) in tenants.iter() {
            for (env, tenant) in env_map.iter() {
                let status = match crate::bridge::invoke_health(self.clone(), tenant.clone()).await {
                    Ok(s) => s,
                    Err(_) => "Unhealthy".to_string(),
                };
                info!("Health check for {} ({}): {}", id, env, status);
                
                // Update session.json status only for the current active perspective
                let current_perspective = self.get_perspective(id);
                if *env == current_perspective {
                    let _ = self.update_session_status(id, &status).await;
                }
            }
        }
        Ok(())
    }

    async fn update_session_status(&self, id: &str, status: &str) -> Result<()> {
        let path = dirs::home_dir().context("No home dir")?.join(".axiom").join("session.json");
        if !path.exists() { return Ok(()); }

        let content = std::fs::read_to_string(&path)?;
        let mut json: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(tomains) = json.get_mut("tomains").and_then(|t| t.as_object_mut()) {
            if let Some(entry) = tomains.get_mut(id).and_then(|e| e.as_object_mut()) {
                entry.insert("status".to_string(), serde_json::json!(status));
                let updated = serde_json::to_string_pretty(&json)?;
                std::fs::write(&path, updated)?;
            }
        }
        Ok(())
    }
}
