use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use tracing::info;

pub struct InfraRegistry {
    pub statuses: Arc<RwLock<HashMap<String, String>>>,
}

impl InfraRegistry {
    pub fn new() -> Self {
        Self {
            statuses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn update_status(&self, id: &str, status: &str) -> anyhow::Result<()> {
        info!("Updating Tomain {} status to {}", id, status);
        self.statuses.write().await.insert(id.to_string(), status.to_string());
        Ok(())
    }
}
