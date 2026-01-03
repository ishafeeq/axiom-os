use anyhow::{Result, Context, anyhow};
use std::sync::Arc;
use crate::supervisor::TenantInstance;
use crate::runtime::WasmSupervisor;
use wasmtime::*;
use wasmtime_wasi::preview1::WasiP1Ctx;
use wasmtime_wasi::WasiCtxBuilder;
use tracing::{info, error, warn};

pub struct HostState {
    pub wasi: WasiP1Ctx,
    pub supervisor: Arc<WasmSupervisor>,
    pub tomain_id: String,
}

pub async fn invoke_reflect(supervisor: Arc<WasmSupervisor>, tenant: Arc<TenantInstance>) -> Result<String> {
    let mut store = create_store(supervisor, tenant.id.clone(), &tenant.engine)?;
    let linker = create_linker(&tenant.engine)?;
    let instance: Instance = linker.instantiate_async(&mut store, &tenant.module).await?;
    
    let func = instance.get_typed_func::<(), u32>(&mut store, "reflect")?;
    let ptr = func.call_async(&mut store, ()).await?;
    
    let memory = instance.get_memory(&mut store, "memory")
        .context("Failed to find memory")?;
        
    let data = memory.data(&store);
    
    let start = ptr as usize;
    let mut end = start;
    while end < data.len() && data[end] != 0 {
        end += 1;
    }
    
    let json = String::from_utf8_lossy(&data[start..end]).to_string();
    Ok(json)
}

pub async fn invoke_health(supervisor: Arc<WasmSupervisor>, tenant: Arc<TenantInstance>) -> Result<String> {
    let mut store = create_store(supervisor, tenant.id.clone(), &tenant.engine)?;
    let linker = create_linker(&tenant.engine)?;
    let _instance: Instance = linker.instantiate_async(&mut store, &tenant.module).await?;
    Ok("Healthy".to_string())
}

pub async fn invoke_call(supervisor: Arc<WasmSupervisor>, tenant: Arc<TenantInstance>, func_name: &str, query_json: String) -> Result<String> {
    let mut store = create_store(supervisor, tenant.id.clone(), &tenant.engine)?;
    let linker = create_linker(&tenant.engine)?;
    let instance: Instance = linker.instantiate_async(&mut store, &tenant.module).await?;
    
    // Name variants to try
    let call_variants = vec![
        format!("__axiom_call_{}", func_name),
        format!("__axiom_call_{}", func_name.replace("-", "_")),
    ];
    let plain_variants = vec![
        func_name.to_string(),
        func_name.replace("-", "_"),
        "axiom_health_check".to_string(),
    ];

    // First, try the __axiom_call_ wrappers that accept (ptr, len) -> u32
    let mut res_ptr = None;
    for variant in &call_variants {
        if let Ok(f) = instance.get_typed_func::<(u32, u32), u32>(&mut store, variant) {
            // Write JSON into Wasm memory
            let memory = instance.get_memory(&mut store, "memory")
                .context("Failed to find memory")?;
            let json_bytes = query_json.as_bytes();
            let json_len = json_bytes.len() as u32;
            
            // Find a safe place to write (after the current data_size)
            let write_offset = memory.data_size(&store) as u32;
            memory.grow(&mut store, 1)?; // Grow by 1 page (64KB) to be safe
            memory.data_mut(&mut store)[write_offset as usize..write_offset as usize + json_bytes.len()]
                .copy_from_slice(json_bytes);
            
            res_ptr = Some(f.call_async(&mut store, (write_offset, json_len)).await?);
            break;
        }
    }

    // Fallback: try plain function names (void or no-arg)
    if res_ptr.is_none() {
        for variant in &plain_variants {
            if let Ok(f) = instance.get_typed_func::<(), u32>(&mut store, variant) {
                res_ptr = Some(f.call_async(&mut store, ()).await?);
                break;
            } else if let Ok(f) = instance.get_typed_func::<(), ()>(&mut store, variant) {
                f.call_async(&mut store, ()).await?;
                res_ptr = Some(0);
                break;
            }
        }
    }

    let res_ptr = res_ptr.context(format!("Function '{}' not found in Wasm module", func_name))?;

    if res_ptr == 0 { return Ok("Success (void/0)".to_string()); }

    let memory = instance.get_memory(&mut store, "memory")
        .context("Failed to find memory")?;
        
    let data = memory.data(&store);
    let start = res_ptr as usize;
    let mut end = start;
    while end < data.len() && data[end] != 0 {
        end += 1;
    }
    
    Ok(String::from_utf8_lossy(&data[start..end]).to_string())
}

fn create_store(supervisor: Arc<WasmSupervisor>, tomain_id: String, engine: &Engine) -> Result<Store<HostState>> {
    let wasi = WasiCtxBuilder::new().inherit_stdout().inherit_stderr().build_p1();
    let state = HostState {
        wasi,
        supervisor,
        tomain_id,
    };
    let mut store = Store::new(engine, state);
    store.set_fuel(1_000_000)?;
    Ok(store)
}

fn create_linker(engine: &Engine) -> Result<Linker<HostState>> {
    let mut linker = Linker::new(engine);
    wasmtime_wasi::preview1::add_to_linker_async(&mut linker, |t: &mut HostState| &mut t.wasi)?;
    
    // Host Functions (Pillar #3: Trusted Identity Loop)
    linker.func_wrap("axiom", "get_family_token", |_caller: Caller<'_, HostState>| -> Result<u32> {
        Ok(0)
    })?;

    // Pillar #9: Egress Guard
    linker.func_wrap_async("axiom", "http_call", |mut caller: Caller<'_, HostState>, (alias_ptr, method_ptr, body_ptr, body_len): (u32, u32, u32, u32)| {
        Box::new(async move {
            let memory = caller.get_export("memory").and_then(|e| e.into_memory()).context("Failed to get memory")?;
            
            // 1. Read alias, method, and body from Wasm memory
            let alias = read_wasm_string(&caller, &memory, alias_ptr as usize)?;
            let method_name = read_wasm_string(&caller, &memory, method_ptr as usize)?.to_uppercase();
            
            let body_bytes = if body_ptr > 0 && body_len > 0 {
                let mut buf = vec![0u8; body_len as usize];
                memory.read(&caller, body_ptr as usize, &mut buf)?;
                Some(buf)
            } else {
                None
            };
            
            let (supervisor, tomain_id) = {
                let state = caller.data();
                (state.supervisor.clone(), state.tomain_id.clone())
            };
            let environment = supervisor.perspective.get(&tomain_id).map(|p| p.value().clone()).unwrap_or_else(|| "GREEN".to_string());
            
            // Pillar #3: Sampling Rate Adjustment
            if environment == "BLUE" {
                info!("📊 [SAMPLING++]: Trace sampling rate increased for BLUE perspective.");
            }
            
            // Pillar #4: Audit Mode (RED)
            if environment == "RED" {
                let audit_entry = format!("HTTP {} {} (Alias: {})", method_name, tomain_id, alias);
                supervisor.audit_log.entry(tomain_id.clone()).or_insert_with(Vec::new).push(audit_entry);
                info!("🔴 [AUDIT]: Recorded state change: HTTP {} to {}", method_name, alias);
            }
            
            // Pillar #6: Security Boundary
            // Ensure target service is promoted to the caller's environment
            if supervisor.manager.get_tenant(&alias, &environment).await.is_none() {
                // Check if it's an external URL (starts with http) or a logical alias
                if !alias.starts_with("http") {
                    warn!("🛑 Security Boundary: Service '{}' is not promoted to {} environment. Call blocked.", alias, environment);
                    return Ok(write_wasm_string(&mut caller, &memory, &format!("Error: Security Boundary: {} not promoted to {}", alias, environment)));
                }
            }

            // 2. Resolve alias to physical URL
            match supervisor.egress.resolve(&tomain_id, &alias, &environment).await {
                Ok(url) => {
                    info!("🚀 Egress Guard: Resolved '{}' -> {} (Method: {}, Tomain: {}, Env: {})", alias, url, method_name, tomain_id, environment);
                    
                    // 3. Downstream Resilience Guards
                    let resilience = supervisor.resilience.clone();
                    
                    // a. Rate Limiting (10 req/sec default for now)
                    if !resilience.traffic.check_downstream(&alias, 10.0) {
                        warn!("⏳ Downstream Rate Limit: Throttling '{}'", alias);
                        return Ok(write_wasm_string(&mut caller, &memory, "Error: Rate Limit Exceeded (429)"));
                    }

                    // b. Circuit Breaker
                    if !resilience.fault.breakers.entry(alias.clone()).or_insert_with(crate::resilience::CircuitBreaker::new).value_mut().should_allow() {
                        warn!("🚨 Downstream Circuit OPEN: Blocking call to '{}'", alias);
                        return Ok(write_wasm_string(&mut caller, &memory, "Error: Circuit Breaker Open"));
                    }

                    // 4. Exponential Backoff Retries (Pillar #2)
                    let mut attempts = 0;
                    let max_retries = 3;
                    let mut last_result: Result<reqwest::Response, anyhow::Error> = Err(anyhow!("Request not started"));

                    while attempts <= max_retries {
                        if attempts > 0 {
                            let delay = 2u64.pow(attempts as u32 - 1);
                            info!("🔁 Retrying '{}' (Attempt {}/3) in {}s...", alias, attempts, delay);
                            tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
                        }

                        // We need to clone the request builder for retries
                        // reqwest::RequestBuilder doesn't implement Clone, so we re-create it
                        let mut retry_req = supervisor.http_client.request(
                            match method_name.as_str() {
                                "POST" => reqwest::Method::POST,
                                "PUT" => reqwest::Method::PUT,
                                "DELETE" => reqwest::Method::DELETE,
                                _ => reqwest::Method::GET,
                            }, 
                            &url
                        );
                        if let Some(token) = resilience.security.get_vault_token(&alias) {
                            retry_req = retry_req.header("Authorization", format!("Bearer {}", token));
                        }
                        if let Some(ref body) = body_bytes {
                            retry_req = retry_req.body(body.clone());
                        }

                        match retry_req.send().await {
                            Ok(resp) if resp.status().is_success() => {
                                let text = resp.text().await.unwrap_or_else(|_| "Error reading body".to_string());
                                resilience.fault.breakers.get_mut(&alias).unwrap().report_success();
                                return Ok(write_wasm_string(&mut caller, &memory, &text));
                            }
                            Ok(resp) if resp.status().is_server_error() => {
                                warn!("⚠️ Transient error ({}) on '{}'. Retrying...", resp.status(), alias);
                                last_result = Err(anyhow!("Server Error: {}", resp.status()));
                            }
                            Ok(resp) => {
                                let text = resp.text().await.unwrap_or_else(|_| "Error reading body".to_string());
                                resilience.fault.breakers.get_mut(&alias).unwrap().report_failure();
                                return Ok(write_wasm_string(&mut caller, &memory, &text));
                            }
                            Err(e) => {
                                warn!("⚠️ Request error: {:?}. Retrying...", e);
                                last_result = Err(anyhow::Error::new(e));
                            }
                        }
                        attempts += 1;
                    }

                    // If max retries exhausted
                    resilience.fault.breakers.get_mut(&alias).unwrap().report_failure();
                    warn!("❌ Max retries exhausted for '{}': {:?}", alias, last_result);
                    Ok(write_wasm_string(&mut caller, &memory, &format!("Error: Downstream FAILED after 3 retries: {:?}", last_result)))
                },
                Err(_) => {
                    warn!("🛑 Egress Guard: Blocking call to unauthorized alias '{}' (Tomain: {})", alias, tomain_id);
                    Ok(0u32) 
                }
            }
        })
    })?;

    // Pillar #1: Database Bridge
    linker.func_wrap_async("axiom", "db_execute", |mut caller: Caller<'_, HostState>, (alias_ptr, query_ptr, query_len): (u32, u32, u32)| {
        Box::new(async move {
            let memory = caller.get_export("memory").and_then(|e| e.into_memory()).context("Failed to get memory")?;
            
            let alias = read_wasm_string(&caller, &memory, alias_ptr as usize)?;
            let query_json = if query_ptr > 0 && query_len > 0 {
                let mut buf = vec![0u8; query_len as usize];
                memory.read(&caller, query_ptr as usize, &mut buf)?;
                String::from_utf8_lossy(&buf).to_string()
            } else {
                return Ok(0u32);
            };

            let (supervisor, tomain_id) = {
                let s = caller.data();
                (s.supervisor.clone(), s.tomain_id.clone())
            };
            let environment = supervisor.perspective.get(&tomain_id).map(|p| p.value().clone()).unwrap_or_else(|| "GREEN".to_string());

            if environment == "RED" {
                let audit_entry = format!("DB_EXECUTE {} (Alias: {})", tomain_id, alias);
                supervisor.audit_log.entry(tomain_id.clone()).or_insert_with(Vec::new).push(audit_entry);
                info!("🔴 [AUDIT]: Recorded state change: DB EXECUTE on {}", alias);
            }
            
            let query: crate::db::AxiomQuery = serde_json::from_str(&query_json).context("Failed to parse AxiomQuery")?;

            if let Some(provider) = supervisor.db_registry.get(&alias) {
                match provider.execute_query(query).await {
                    Ok(resp) => {
                        let res_json = serde_json::to_string(&resp).unwrap_or_default();
                        Ok(write_wasm_string(&mut caller, &memory, &res_json))
                    }
                    Err(e) => {
                        error!("DB Egress call FAILED (Alias: {}): {:?}", alias, e);
                        Ok(0u32)
                    }
                }
            } else {
                warn!("🛑 DB Guard: No provider found for alias '{}'", alias);
                Ok(0u32)
            }
        })
    })?;

    // Pillar #3: SDK Visibility
    linker.func_wrap_async("axiom", "axiom_health_status", |mut caller: Caller<'_, HostState>, (alias_ptr,): (u32,)| {
        Box::new(async move {
            let memory = caller.get_export("memory").and_then(|e| e.into_memory()).context("Failed to get memory")?;
            let alias = read_wasm_string(&caller, &memory, alias_ptr as usize)?;
            
            let supervisor = caller.data().supervisor.clone();
            let state = supervisor.resilience.fault.get_status(&alias);
            
            let state_str = format!("{:?}", state);
            Ok(write_wasm_string(&mut caller, &memory, &state_str))
        })
    })?;

    // Pillar #3: Native Logging
    linker.func_wrap("axiom", "axiom_log", |mut caller: Caller<'_, HostState>, ptr: u32, len: u32, level: u32| {
        let memory = caller.get_export("memory").and_then(|e| e.into_memory()).context("Failed to get memory")?;
        let data = memory.data(&caller);
        let start = ptr as usize;
        let end = start + len as usize;
        
        if end > data.len() {
            return Err(anyhow!("Log pointer out of bounds"));
        }
        
        let msg = String::from_utf8_lossy(&data[start..end]).to_string();
        let tomain_id = &caller.data().tomain_id;
        
        match level {
            0 => error!(tomain_id = %tomain_id, "{}", msg),
            1 => warn!(tomain_id = %tomain_id, "{}", msg),
            2 => info!(tomain_id = %tomain_id, "{}", msg),
            3 => tracing::debug!(tomain_id = %tomain_id, "{}", msg),
            _ => tracing::trace!(tomain_id = %tomain_id, "{}", msg),
        }
        Ok(())
    })?;

    Ok(linker)
}

fn write_wasm_string(caller: &mut Caller<'_, HostState>, memory: &Memory, text: &str) -> u32 {
    let res_bytes = format!("{}\0", text).into_bytes();
    let write_offset = memory.data_size(&mut *caller);
    let pages_needed = (res_bytes.len() / 65536) + 1;
    let _ = memory.grow(&mut *caller, pages_needed as u64);
    
    if let Err(e) = memory.write(&mut *caller, write_offset, &res_bytes) {
        warn!("Failed to write to Wasm memory: {}", e);
        return 0;
    }
    write_offset as u32
}

fn read_wasm_string(caller: &impl AsContext, memory: &Memory, ptr: usize) -> Result<String> {
    let mut data = vec![0u8; 256];
    let _ = memory.read(caller, ptr, &mut data);
    let mut end = 0;
    while end < data.len() && data[end] != 0 {
        end += 1;
    }
    Ok(String::from_utf8_lossy(&data[..end]).to_string())
}
