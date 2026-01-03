use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions, Column};
use std::sync::Arc;
use tracing::{info, error, instrument};
use anyhow::{Result, Context};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct AxiomQuery {
    pub sql: String,
    pub params: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AxiomResponse {
    pub rows: Vec<serde_json::Value>,
    pub affected_rows: u64,
}

#[async_trait]
pub trait AxiomDatabaseProvider: Send + Sync {
    async fn execute_query(&self, query: AxiomQuery) -> Result<AxiomResponse>;
    #[allow(dead_code)]
    async fn health_check(&self) -> Result<String>;
    #[allow(dead_code)]
    fn provider_name(&self) -> &'static str;
}

pub struct PostgresAdapter {
    pool: Pool<Postgres>,
}

impl PostgresAdapter {
    pub async fn new(url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await
            .context("Failed to connect to Postgres")?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl AxiomDatabaseProvider for PostgresAdapter {
    #[instrument(skip(self, query), fields(db.system = "postgres", db.operation = "query"))]
    async fn execute_query(&self, query: AxiomQuery) -> Result<AxiomResponse> {
        info!("Executing Postgres query: {}", query.sql);
        
        // Basic input sanitization (Pillar #9)
        // In a real impl, we'd use prepared statements correctly.
        // For this demo, we'll use sqlx's query functionality.
        
        let mut q = sqlx::query::<Postgres>(&query.sql);
        for param in &query.params {
            match param {
                Value::String(s) => q = q.bind(s),
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() { q = q.bind(i); }
                    else if let Some(f) = n.as_f64() { q = q.bind(f); }
                },
                Value::Bool(b) => q = q.bind(b),
                _ => q = q.bind(param.to_string()),
            }
        }

        use sqlx::Row;
        let rows = sqlx::query::<Postgres>(&query.sql)
            .fetch_all(&self.pool)
            .await?;

        let mut row_values = Vec::new();
        for row in rows {
            let mut map = serde_json::Map::new();
            for column in row.columns() {
                let name = column.name();
                // Extremely simplified type mapping for the demo
                let val: serde_json::Value = if let Ok(s) = row.try_get::<String, _>(name) {
                    serde_json::Value::String(s)
                } else if let Ok(i) = row.try_get::<i64, _>(name) {
                    serde_json::Value::Number(i.into())
                } else {
                    serde_json::Value::Null
                };
                map.insert(name.to_string(), val);
            }
            row_values.push(serde_json::Value::Object(map));
        }

        Ok(AxiomResponse {
            rows: row_values,
            affected_rows: 0,
        })
    }

    async fn health_check(&self) -> Result<String> {
        sqlx::query::<Postgres>("SELECT 1").execute(&self.pool).await?;
        Ok("Healthy".to_string())
    }

    fn provider_name(&self) -> &'static str {
        "postgres"
    }
}

pub struct MockAdapter;

#[async_trait]
impl AxiomDatabaseProvider for MockAdapter {
    async fn execute_query(&self, query: AxiomQuery) -> Result<AxiomResponse> {
        info!("🎭 Mock DB executing: {}", query.sql);
        Ok(AxiomResponse {
            rows: vec![serde_json::json!({"id": 1, "name": "Mock Item"})],
            affected_rows: 1,
        })
    }
    async fn health_check(&self) -> Result<String> { Ok("Mock Healthy".to_string()) }
    fn provider_name(&self) -> &'static str { "mock" }
}

pub struct DatabaseRegistry {
    pub providers: Arc<dashmap::DashMap<String, Arc<dyn AxiomDatabaseProvider>>>,
}

impl DatabaseRegistry {
    pub fn new() -> Self {
        Self {
            providers: Arc::new(dashmap::DashMap::new()),
        }
    }

    pub fn register(&self, alias: String, provider: Arc<dyn AxiomDatabaseProvider>) {
        self.providers.insert(alias, provider);
    }

    pub fn get(&self, alias: &str) -> Option<Arc<dyn AxiomDatabaseProvider>> {
        self.providers.get(alias).map(|p| p.value().clone())
    }

    pub async fn reload_from_registry(&self) -> Result<()> {
        let path = dirs::home_dir()
            .unwrap_or_default()
            .join(".axiom")
            .join("session.json");

        if let Ok(content) = std::fs::read_to_string(&path) {
            let json: Value = serde_json::from_str(&content)?;
            if let Some(db_configs) = json.get("databases").and_then(|d| d.as_object()) {
                for (alias, config) in db_configs {
                    let provider = config.get("provider").and_then(|p| p.as_str()).unwrap_or("postgres");
                    let url = config.get("url").and_then(|u| u.as_str()).unwrap_or("");
                    
                    if provider == "postgres" && !url.is_empty() {
                        match PostgresAdapter::new(url).await {
                            Ok(adapter) => {
                                self.register(alias.clone(), Arc::new(adapter));
                                info!("Registered DB provider: {} (postgres)", alias);
                            }
                            Err(e) => error!("Failed to initialize DB provider {}: {}", alias, e),
                        }
                    } else if provider == "mock" {
                        self.register(alias.clone(), Arc::new(MockAdapter));
                        info!("Registered DB provider: {} (mock)", alias);
                    }
                }
            }
        }
        Ok(())
    }
}
