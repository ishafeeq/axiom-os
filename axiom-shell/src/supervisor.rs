use anyhow::{Result, Context};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use wasmtime::*;
use tracing::info;

pub struct TenantInstance {
    pub id: String,
    pub engine: Engine,
    pub module: Module,
}

pub struct TenantManager {
    /// tomain_id -> { env -> TenantInstance }
    pub tenants: Arc<RwLock<HashMap<String, HashMap<String, Arc<TenantInstance>>>>>,
}

impl TenantManager {
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_engine(&self) -> Result<Engine> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        config.consume_fuel(true);
        Engine::new(&config)
    }

    pub async fn register_tenant(&self, id: &str, env: &str, wasm_bytes: &[u8]) -> Result<()> {
        let engine = self.create_engine()?;
        let module = Module::new(&engine, wasm_bytes).context("Failed to load Wasm module")?;
        
        let instance = Arc::new(TenantInstance {
            id: id.to_string(),
            engine,
            module,
        });

        let mut all_tenants = self.tenants.write().await;
        let tenant_envs = all_tenants.entry(id.to_string()).or_insert_with(HashMap::new);
        tenant_envs.insert(env.to_uppercase(), instance);
        
        info!("Tenant registered: {} in {} slot", id, env);
        Ok(())
    }

    pub async fn get_tenant(&self, id: &str, env: &str) -> Option<Arc<TenantInstance>> {
        let all_tenants = self.tenants.read().await;
        all_tenants.get(id)?.get(&env.to_uppercase()).cloned()
    }

    pub async fn remove_tenant(&self, id: &str, env: &str) -> Result<()> {
        let mut all_tenants = self.tenants.write().await;
        if let Some(tenant_envs) = all_tenants.get_mut(id) {
            tenant_envs.remove(&env.to_uppercase());
            if tenant_envs.is_empty() {
                all_tenants.remove(id);
            }
        }
        info!("Tenant retired: {} from {} slot", id, env);
        Ok(())
    }
}
