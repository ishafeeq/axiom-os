use anyhow::Result;
use std::sync::Arc;
use crate::runtime::WasmSupervisor;
use tracing::info;

pub async fn watch_sessions(_sv: Arc<WasmSupervisor>) -> Result<()> {
    info!("Session watcher active (standing by for environment changes)...");
    // In a full implementation, this would use 'notify' to watch ~/.axiom/
    Ok(())
}
