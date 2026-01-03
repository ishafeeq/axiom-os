use std::sync::Arc;
use dashmap::DashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use anyhow::{Result, anyhow};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

// --- Security Pillar #9 ---

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub struct SecurityManager {
    /// Public keys for JWT validation (tomain_id -> PEM)
    pub public_keys: Arc<DashMap<String, String>>,
    /// Vault for downstream tokens (alias -> token)
    pub vault: Arc<DashMap<String, String>>,
}

impl SecurityManager {
    pub fn new() -> Self {
        Self {
            public_keys: Arc::new(DashMap::new()),
            vault: Arc::new(DashMap::new()),
        }
    }

    pub fn validate_jwt(&self, tomain_id: &str, token: &str) -> Result<()> {
        let pem = self.public_keys.get(tomain_id)
            .ok_or_else(|| anyhow!("No public key found for tomain: {}", tomain_id))?;
            
        let key = DecodingKey::from_rsa_pem(pem.as_bytes())?;
        let validation = Validation::new(Algorithm::RS256);
        decode::<Claims>(token, &key, &validation)?;
        Ok(())
    }

    pub fn get_vault_token(&self, alias: &str) -> Option<String> {
        self.vault.get(alias).map(|t| t.value().clone())
    }
}

// --- Traffic Pillar #1 ---

pub struct TokenBucket {
    pub capacity: f64,
    pub tokens: f64,
    pub fill_rate: f64, // tokens per second
    pub last_filled: DateTime<Utc>,
}

impl TokenBucket {
    pub fn new(rate: f64) -> Self {
        Self {
            capacity: rate,
            tokens: rate,
            fill_rate: rate,
            last_filled: Utc::now(),
        }
    }

    pub fn try_consume(&mut self) -> bool {
        let now = Utc::now();
        let elapsed = (now - self.last_filled).num_milliseconds() as f64 / 1000.0;
        self.tokens = (self.tokens + elapsed * self.fill_rate).min(self.capacity);
        self.last_filled = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

pub struct TrafficController {
    /// Upstream rate limiting (tomain_id -> bucket)
    pub upstream_buckets: Arc<DashMap<String, TokenBucket>>,
    /// Downstream rate limiting (alias -> bucket)
    pub downstream_buckets: Arc<DashMap<String, TokenBucket>>,
}

impl TrafficController {
    pub fn new() -> Self {
        Self {
            upstream_buckets: Arc::new(DashMap::new()),
            downstream_buckets: Arc::new(DashMap::new()),
        }
    }

    pub fn check_upstream(&self, tomain_id: &str, limit_per_sec: f64) -> bool {
        let mut bucket = self.upstream_buckets.entry(tomain_id.to_string())
            .or_insert_with(|| TokenBucket::new(limit_per_sec));
        bucket.try_consume()
    }

    pub fn check_downstream(&self, alias: &str, limit_per_sec: f64) -> bool {
        let mut bucket = self.downstream_buckets.entry(alias.to_string())
            .or_insert_with(|| TokenBucket::new(limit_per_sec));
        bucket.try_consume()
    }
}

// --- Fault Tolerance Pillar #2 ---

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    pub state: CircuitState,
    pub failure_count: u32,
    pub last_failure: Option<DateTime<Utc>>,
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure: None,
        }
    }

    pub fn report_success(&mut self) {
        self.state = CircuitState::Closed;
        self.failure_count = 0;
    }

    pub fn report_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Utc::now());
        if self.failure_count >= 5 {
            self.state = CircuitState::Open;
            warn!("🚨 Circuit Breaker OPENED after 5 failures.");
        }
    }

    pub fn should_allow(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                let now = Utc::now();
                if let Some(last) = self.last_failure {
                    if (now - last).num_seconds() > 30 {
                        self.state = CircuitState::HalfOpen;
                        info!("🔄 Circuit Breaker HALF-OPEN (Testing...).");
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }
}

pub struct FaultTolerance {
    pub breakers: Arc<DashMap<String, CircuitBreaker>>,
}

impl FaultTolerance {
    pub fn new() -> Self {
        Self {
            breakers: Arc::new(DashMap::new()),
        }
    }

    pub fn get_status(&self, alias: &str) -> CircuitState {
        self.breakers.get(alias)
            .map(|b| b.state)
            .unwrap_or(CircuitState::Closed)
    }
}

// --- Resilience Manager ---

pub struct ResilienceManager {
    pub security: SecurityManager,
    pub traffic: TrafficController,
    pub fault: FaultTolerance,
}

impl ResilienceManager {
    pub fn new() -> Self {
        Self {
            security: SecurityManager::new(),
            traffic: TrafficController::new(),
            fault: FaultTolerance::new(),
        }
    }

    pub async fn reload_from_registry(&self) -> Result<()> {
        let path = dirs::home_dir()
            .unwrap_or_default()
            .join(".axiom")
            .join("session.json");

        if let Ok(content) = std::fs::read_to_string(&path) {
            let json: serde_json::Value = serde_json::from_str(&content)?;
            
            // Clear existing state for a fresh reload
            self.security.public_keys.clear();
            self.security.vault.clear();
            self.traffic.upstream_buckets.clear();
            self.traffic.downstream_buckets.clear();

            // 1. Load Public Keys (for Upstream Auth)
            if let Some(keys) = json.get("public_keys").and_then(|k| k.as_object()) {
                for (tomain_id, key) in keys {
                    if let Some(key_str) = key.as_str() {
                        self.security.public_keys.insert(tomain_id.clone(), key_str.to_string());
                        info!("🔐 Loaded public key for tomain: {}", tomain_id);
                    }
                }
            }

            // 2. Load Vault Tokens (for Downstream Auth)
            if let Some(vault) = json.get("vault").and_then(|v| v.as_object()) {
                for (alias, token) in vault {
                    if let Some(token_str) = token.as_str() {
                        self.security.vault.insert(alias.clone(), token_str.to_string());
                        info!("🔑 Loaded vault token for alias: {}", alias);
                    }
                }
            }

            // 3. Load Rate Limits
            if let Some(limits) = json.get("rate_limits").and_then(|l| l.as_object()) {
                if let Some(upstream) = limits.get("upstream").and_then(|u| u.as_object()) {
                    for (tomain_id, limit) in upstream {
                        if let Some(l) = limit.as_f64() {
                            self.traffic.upstream_buckets.insert(tomain_id.clone(), TokenBucket::new(l));
                            info!("🚦 Set upstream rate limit for {}: {} req/sec", tomain_id, l);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
