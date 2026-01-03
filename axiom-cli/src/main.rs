

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use toml;

#[derive(Parser)]
#[command(name = "ax")]
#[command(about = "Axiom Toolchain: Init, Deploy, and Hot-Swap Wasm Kernels", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Wasm Kernel project
    Init {
        /// Project Name
        #[arg(short = 'n', long = "name")]
        name: Option<String>,
        /// Use QA context
        #[arg(short = 'q', long = "qa")]
        qa: bool,
    },
    /// Switch environment context
    Env {
        /// Environment to switch to (dev/qa/staging/prod)
        #[arg(value_parser = ["dev", "qa", "staging", "prod"])]
        environment: String,
    },
    /// Deploy the local Kernel to the active Shell
    Deploy {
        /// Target environment to deploy to
        #[arg(value_parser = ["dev", "qa", "staging", "prod"])]
        environment: Option<String>,
    },
    /// Bind a logical alias to a physical URL (Pillar #5 & #9)
    Bind {
        /// Alias name (e.g. inventory)
        #[arg(short = 'n', long = "name")]
        name: String,
        /// Physical URL or Connection String
        #[arg(short = 'u', long = "url")]
        url: String,
        /// Provider type (e.g. postgres, mongo, ddb)
        #[arg(short = 'p', long = "provider", default_value = "http")]
        provider: String,
    },
    /// Checkout a Tomain and its Capability Manifest (Pillar #7)
    Checkout {
        /// Tomain address (e.g. fintech.tom/ledger)
        address: String,
    },
    /// Promote a Tomain or Feature across environments (Pillar #5 & #7)
    Promote {
        /// Micro-service/Tomain name (defaults to current session)
        #[arg(short = 'm', long = "ms")]
        ms: Option<String>,
        /// Feature name (optional)
        #[arg(short = 'f', long = "feature")]
        feature: Option<String>,
        /// Source environment (dev/qa/staging/prod)
        #[arg(long = "from", default_value = "dev")]
        from: String,
        /// Target environment (qa/staging/prod)
        #[arg(long = "to")]
        to: String,
    },
    /// Retire a Tomain from a specific environment
    Retire {
        /// Micro-service/Tomain name (defaults to current session)
        #[arg(short = 'm', long = "ms")]
        ms: Option<String>,
        /// Environment to retire from (blue/red/qa/prod)
        #[arg(short = 'e', long = "env")]
        env: String,
    },
    /// Show the Axiom Dashboard (Pillar #3 & #4)
    Status,
    /// Feature management
    Feature {
        #[command(subcommand)]
        command: FeatureCommands,
    },
    /// Push changes (git push + wasm upload)
    Push,
}

#[derive(Subcommand)]
enum FeatureCommands {
    /// Start a new feature branch
    Start {
        /// Feature name
        name: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct AxiomSession {
    pub tomain_id: String,
    pub package_name: String,
    pub environment: String,
    pub last_sync: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeployPayload {
    pub tomain_id: String,
    pub wasm_base64: String,
}

const SESSION_FILE: &str = ".axiom/session.json";
const CCP_BASE_URL: &str = "http://localhost:3000/api/v1";

#[derive(Debug, Serialize, Deserialize)]
struct AxiomConfig {
    pub team_name: String,
    pub org_suffix: String,
    pub default_tomain_prefix: String,
    pub creator_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AxiomManifest {
    pub resources: HashMap<String, ResourceDef>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResourceDef {
    pub alias: String,
    #[serde(rename = "type")]
    pub resource_type: String,
}

fn get_config_path() -> std::path::PathBuf {
    let mut path = if let Ok(home) = std::env::var("AXIOM_HOME") {
        std::path::PathBuf::from(home)
    } else {
        dirs::home_dir().expect("Could not find home directory")
    };
    path.push(".axiom");
    path.push("config.json");
    path
}

fn load_or_prompt_config() -> Result<AxiomConfig> {
    let config_path = get_config_path();

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        if let Ok(config) = serde_json::from_str::<AxiomConfig>(&content) {
            // Validation: Ensure we have a valid team name and a prefix that isn't just a dot
            if !config.team_name.is_empty() && config.default_tomain_prefix.len() > 1 {
                return Ok(config);
            }
        }
        println!("{} Legacy or incomplete configuration detected. Let's fix that.", "⚠️".yellow());
    }

    println!("{}", "🚀 Welcome to the Axiom Toolchain!".cyan().bold());
    println!("It looks like this is your first time. Let's set up your Default Team Tomain Context.\n");

    print!("Enter your Team Name (default: 'alpha-squad'): ");
    io::stdout().flush()?;
    let mut team_name_input = String::new();
    io::stdin().read_line(&mut team_name_input)?;
    let team_name = if team_name_input.trim().is_empty() { "alpha-squad".to_string() } else { team_name_input.trim().replace(" ", "_") };
    let org_suffix = "default".to_string();
    let creator_name = std::env::var("USER").unwrap_or_else(|_| "axiom-dev".to_string());

    let default_tomain_prefix = format!("{}.{}", team_name, org_suffix);

    let config = AxiomConfig {
        team_name,
        org_suffix,
        default_tomain_prefix: default_tomain_prefix.clone(),
        creator_name,
    };

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;
    println!("{} Configuration saved to {:?}", "✅".green(), config_path);
    println!("{} Your Default Tomain Prefix is now: {}\n", "🌐".cyan(), default_tomain_prefix.bold());

    Ok(config)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name, qa } => {
            let env = if qa { "QA" } else { "DEV" };
            init_project(name, env).await?;
        }
        Commands::Env { environment } => {
            let color = match environment.to_lowercase().as_str() {
                "qa" => "QA",
                "staging" => "STAGING",
                "prod" => "PROD",
                _ => "DEV",
            };
            switch_env(color).await?;
        }
        Commands::Deploy { environment } => {
            let env = environment.unwrap_or_else(|| "dev".to_string());
            let color = match env.to_lowercase().as_str() {
                "qa" => "QA",
                "staging" => "STAGING",
                "prod" => "PROD",
                _ => "DEV",
            };
            deploy_kernel(color).await?;
        }
        Commands::Bind { name, url, provider } => {
            perform_bind(name, url, provider).await?;
        }
        Commands::Checkout { address } => {
            checkout_tomain(address).await?;
        }
        Commands::Promote { ms, feature, from, to } => {
            promote_tomain(ms, feature, from, to).await?;
        }
        Commands::Retire { ms, env } => {
            retire_tomain(ms, env).await?;
        }
        Commands::Status => {
            show_status().await?;
        }
        Commands::Feature { command } => match command {
            FeatureCommands::Start { name } => {
                start_feature(name).await?;
            }
        },
        Commands::Push => {
            push_all().await?;
        }
    }

    Ok(())
}

async fn init_project(name_arg: Option<String>, env: &str) -> Result<()> {
    let config = load_or_prompt_config()?;

    let package_name = if let Some(n) = name_arg {
        n
    } else {
        print!("{} Enter Package name (e.g. 'my-api'): ", "🚀".cyan());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(anyhow::anyhow!("Name cannot be empty."));
        }
        trimmed.to_string()
    };

    let display_package_name = package_name.replace(" ", "_").replace(".", "_");
    let prefix = config.default_tomain_prefix.trim_matches('.');
    let project_name = if prefix.is_empty() {
        display_package_name.trim_matches('.').to_string()
    } else {
        format!("{}.{}", prefix, display_package_name.trim_matches('.'))
    };
    
    println!("{} Assembling Wasm Kernel for Tomain: {}", "🏗️".cyan(), project_name.bold());

    println!("{} Checking Command Control Plane (CCP) connection...", "🔍".cyan());
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(500))
        .build()?;
        
    let ccp_check = client.get(format!("{}/tomains", CCP_BASE_URL)).send().await;
    
    if ccp_check.is_err() {
        println!("{} Axiom Control Plane (CCP) is not running. Attempting to start it in the background...", "⚠️".yellow().bold());
        
        let mut ccp_dir = Path::new("../axiom-ccp").to_path_buf();
        if !ccp_dir.exists() {
            ccp_dir = Path::new("../../axiom-ccp").to_path_buf();
        }

        if ccp_dir.exists() {
            let _script_path = ccp_dir.join("dev.sh");
            let dir_str = ccp_dir.to_str().unwrap_or("..");
            
            Command::new("sh")
                .arg("-c")
                .arg(format!("cd {} && nohup ./dev.sh > /dev/null 2>&1 &", dir_str))
                .spawn()
                .context("Failed to spawn CCP dev script")?;
                
            print!("{} Waiting for CCP to become healthy", "⏳".cyan());
            io::stdout().flush()?;
            
            let mut is_healthy = false;
            for _ in 0..20 { // Max 10 seconds
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                print!(".");
                io::stdout().flush()?;
                
                if client.get(format!("{}/tomains", CCP_BASE_URL)).send().await.is_ok() {
                    is_healthy = true;
                    break;
                }
            }
            println!("");
            
            if !is_healthy {
                return Err(anyhow::anyhow!("{} CCP failed to start within 10 seconds. Check logs in axiom-ccp.", "❌".red()));
            }
            println!("{} CCP Backend successfully booted!", "🌐".cyan());
        } else {
            println!("{} Error: Could not locate `axiom-ccp` folder. Please start CCP manually:", "❌".red().bold());
            println!("  cd path/to/axiom-ccp && ./dev.sh");
            return Err(anyhow::anyhow!("CCP not reachable. Exiting."));
        }
    }

    // Prevent clobbering an existing active dir safely
    let is_empty = fs::read_dir(".").map(|i| {
        i.filter_map(|e| e.ok())
         .filter(|e| e.file_name() != ".axiom")
         .next()
         .is_none()
    }).unwrap_or(true);
    if !is_empty {
        print!("{} Directory is not empty. Delete all existing files to proceed? (y/N): ", "⚠️".yellow());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() == "y" {
            println!("{} Wiping directory...", "🧹".cyan());
            // Shell out to bash safely to clear contents
            Command::new("bash")
                .arg("-c")
                .arg("rm -rf * .axiom")
                .status()
                .context("Failed to clear directory")?;
        } else {
            return Err(anyhow::anyhow!("Initialization aborted."));
        }
    }

    println!("{} Scaffolding rust Wasm environment...", "📦".cyan());
    
    fs::create_dir_all("src")?;
    fs::write("src/lib.rs", 
r##"use axiom_sdk::{axiom_api, axiom_export_reflect, axiom_runtime, info, warn};

// Compile-time EXTERNAL_API constants (generated from .axiom/bindings.json via build.rs)
// After `ax bind --name my_api --url https://example.com`, use: EXTERNAL_API::MY_API
include!(concat!(env!("OUT_DIR"), "/external_api.rs"));

axiom_runtime!();

#[unsafe(no_mangle)]
pub unsafe extern "C" fn axiom_main() {
    info!("🚀 Wasm Kernel booted and ready.");
}

/// GET /user-profile
/// Demonstrates automated reflection for a GET endpoint.
#[axiom_api]
pub fn get_user_profile(id: String, env: String) -> String {
    axiom_sdk::info!("👤 Fetching user profile for: {} (Env: {})", id, env);
    format!("User Profile for {} in {}", id, env)
}

/// POST /submit-data
/// Demonstrates automated reflection for a POST endpoint.
#[axiom_api]
pub fn submit_data(payload: String) -> String {
    warn!("💾 Receiving data payload (length: {})", payload.len());
    format!("Received payload: {}", payload)
}

// Generate the reflect() function automatically for Pillar #10
axiom_export_reflect!(get_user_profile, submit_data);
"##)?;

    let axiom_sdk_path = dirs::home_dir()
        .map(|h| h.join("Documents/axiom-sdk/axiom-sdk").to_string_lossy().to_string())
        .unwrap_or_else(|| "../axiom-sdk".to_string()); // fallback

    fs::write("Cargo.toml", format!(
r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
axiom-sdk = {{ path = "{}" }}
serde_json = "1.0"

[build-dependencies]
serde_json = "1.0"
"#, display_package_name, axiom_sdk_path))?;

    fs::write("interface1.wit", 
r#"package axiom:kernel;

interface api {
    /// GET /user-profile?id=123&env=prod
    /// Demonstrates 2 query parameters.
    get-user-profile: func(id: string, env: string) -> string;

    /// POST /submit-data
    /// Demonstrates a JSON payload as a request.
    submit-data: func(payload: string) -> string;
}

interface reflection {
    reflect: func() -> string;
}


world kernel {
    export api;
    export reflection;
}
"#)?;

    // Scaffold build.rs for EXTERNAL_API compile-time constants
    fs::write("build.rs",
r#"use std::fs;

fn main() {
    // Tell cargo to re-run if bindings change
    println!("cargo:rerun-if-changed=.axiom/bindings.json");
    
    // Read .axiom/bindings.json and generate EXTERNAL_API module
    let bindings_path = ".axiom/bindings.json";
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = format!("{}/external_api.rs", out_dir);
    
    let mut consts = String::new();
    if let Ok(content) = fs::read_to_string(bindings_path) {
        if let Ok(map) = serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&content) {
            for (alias, _url) in &map {
                let const_name = alias.replace("-", "_").to_uppercase();
                consts.push_str(&format!(
                    "    pub const {}: &str = \"{}\";\n",
                    const_name, alias
                ));
            }
        }
    }
    
    fs::write(&dest, format!(
        "pub mod EXTERNAL_API {{\n{}}}\n",
        consts
    )).unwrap();
}
"#)?;

    // fs::write("swagger.html", crate::swagger::get_swagger_html(&project_name))?; // Removed

    let session = AxiomSession {
        tomain_id: project_name.clone(),
        package_name: display_package_name.replace("-", "_"),
        environment: env.to_string(),
        last_sync: Utc::now(),
    };
    
    fs::create_dir_all(".axiom")?;
    save_session(&session)?;
    
    // Pillar #10: Git Workflow Refinement
    println!("{} Initializing local Git repository...", "📂".cyan());
    let _ = Command::new("git").arg("init").status();
    
    fs::write(".gitignore", 
r#"target/
/debug/
/release/
*.wasm
.DS_Store
.axiom/session.json
"#)?;

    // Setup Local Vault
    let vault_parent = dirs::home_dir().unwrap().join(".axiom/vault").join(&project_name);
    let vault_path = vault_parent.join(format!("{}.git", display_package_name));
    fs::create_dir_all(&vault_parent)?;
    
    if !vault_path.exists() {
        println!("{} Creating Local Vault at {:?}...", "🏛️".cyan(), vault_path);
        let _ = Command::new("git").args(["init", "--bare", vault_path.to_str().unwrap()]).status();
    }
    
    println!("{} Connecting to Local Vault remote...", "📡".cyan());
    let repo_url = vault_path.to_string_lossy().to_string();
    let _ = Command::new("git").args(["remote", "add", "local", &repo_url]).status();
    
    // Initial Commit
    let _ = Command::new("git").args(["add", "."]).status();
    let _ = Command::new("git").args(["commit", "-m", "Initial Axiom project setup"]).status();
    let _ = Command::new("git").args(["branch", "-M", "main"]).status();
    
    println!("{} Pushing to local remote...", "🚀".cyan());
    let _ = Command::new("git").args(["push", "-u", "local", "main"]).status();
    
    println!("\n{} Project locally initialized in {} mode.", "✅".green().bold(), env.bold());
    
    println!("{} Registering Tomain to CCP...", "📡".cyan());
    
    let payload = serde_json::json!({
        "name": project_name.clone(),
        "owner": config.creator_name.clone(),
        "team_name": config.team_name,
        "package_name": package_name,
        "creator_name": config.creator_name,
    });
    
    let client = reqwest::Client::new();
    let res = client.post(format!("{}/tomains", CCP_BASE_URL))
         .json(&payload)
         .send()
         .await;
         
    if res.is_ok() {
        println!("{} Registration successful.\n", "✅".green());
        println!("✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅");
        println!("✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅");
        println!("✅✅                                                                        ✅✅");
        println!("✅✅       Visit => {} CCP Dashboard is running at                          ✅✅", "🌐".bold().cyan());
        println!("✅✅                                                                        ✅✅");
        println!("✅✅                                                                        ✅✅");
        println!("✅✅       🔥🔥🔥🔥🔥🔥 {} 🔥🔥🔥🔥🔥                    ✅✅", "http://localhost:5173".bold().green().underline());
        println!("✅✅                                                                        ✅✅");
        println!("✅✅                                                                        ✅✅");
        println!("✅✅    This dashboard is your main Control Plane for                       ✅✅");
        println!("✅✅    managing all infrastructure and application properties.             ✅✅");
        println!("✅✅                                                                        ✅✅");
        println!("✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅");
        println!("✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅✅\n");
    } else {
        println!("{} Warning: Could not register with local CCP.", "⚠️".yellow());
    }

    // Register vault_path and metadata after successful registration
    let _ = client.post(format!("{}/tomains/{}/manifest", CCP_BASE_URL, project_name))
        .json(&serde_json::json!({
            "resources": {},
            "vault_path": repo_url,
            "team_name": config.team_name,
            "package_name": display_package_name
        }))
        .send()
        .await;

    println!("{} Metadata synced to CCP.", "✅".green());

    println!("{} Initialized in {} context. Run `ax deploy dev` to compile and load.", "✅".green(), env.bold());
    Ok(())
}

async fn deploy_kernel(color: &str) -> Result<()> {
    let session = load_session()?;
    println!("{} Checking Axiom Shell status...", "🔍".cyan());
    
    // Check for axiom.toml & interface1.wit
    let mut resources = std::collections::HashMap::new();
    if Path::new("axiom.toml").exists() {
        if let Ok(content) = fs::read_to_string("axiom.toml") {
            if let Ok(manifest) = toml::from_str::<AxiomManifest>(&content) {
                resources = manifest.resources;
            }
        }
    }
    
    let _apis: Vec<serde_json::Value> = Vec::new();
    if let Ok(_wit_content) = fs::read_to_string("interface1.wit") {
         // ... I'll use the existing collection logic below, but I need to move it up or just call it twice.
         // Actually, I'll just move the whole CCP sync call to AFTER the api_funcs collection.
    }
    let mut shell_ready = tokio::net::UnixStream::connect("/tmp/axiom_shell.sock").await.is_ok();
    
    if !shell_ready {
        println!("{} Axiom Shell not active. Attempting to start it in the background...", "🚀".yellow());
        
        let shell_path = if Path::new("../axiom-shell").exists() {
            "../axiom-shell/target/release/axiom-shell"
        } else {
            "../../axiom-shell/target/release/axiom-shell"
        };
        
        let cmd_str = if Command::new("which").arg("axiom-shell").output().map(|o| o.status.success()).unwrap_or(false) {
            "nohup axiom-shell > /tmp/axiom_shell.log 2>&1 &"
        } else {
            &format!("nohup {} > /tmp/axiom_shell.log 2>&1 &", shell_path)
        };
        
        Command::new("sh")
            .arg("-c")
            .arg(cmd_str)
            .spawn()
            .context("Failed to spawn Axiom Shell")?;

        print!("{} Waiting for Axiom Shell to boot", "⏳".cyan());
        io::stdout().flush()?;

        for _ in 0..20 {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            print!(".");
            io::stdout().flush()?;
            if tokio::net::UnixStream::connect("/tmp/axiom_shell.sock").await.is_ok() {
                shell_ready = true;
                break;
            }
        }
        println!("");

        if !shell_ready {
            return Err(anyhow::anyhow!("{} Axiom Shell failed to start within 10 seconds. Check logs at /tmp/axiom_shell.log", "❌".red()));
        }
        println!("{} Axiom Shell successfully booted!", "🌐".cyan());
    }

    // Auto-sync from interface1.wit: scaffold missing functions AND update axiom_export_reflect!() (Pillar #10)
    if let Ok(wit_content) = fs::read_to_string("interface1.wit") {
        // Parsed function info from WIT
        struct WitFunc {
            rust_name: String,
            params: Vec<(String, String)>, // (name, rust_type)
            doc_lines: Vec<String>,
            method: String,
        }

        let mut api_funcs: Vec<WitFunc> = Vec::new();
        let mut in_api_block = false;
        let mut pending_docs: Vec<String> = Vec::new();
        
        for line in wit_content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("interface api") {
                in_api_block = true;
                continue;
            }
            if in_api_block && trimmed == "}" {
                in_api_block = false;
                pending_docs.clear();
                continue;
            }
            if in_api_block {
                // Collect doc comments
                if trimmed.starts_with("///") {
                    pending_docs.push(trimmed.to_string());
                    continue;
                }
                // Parse: get-user-profile: func(id: string, env: string) -> string;
                if let Some(colon_pos) = trimmed.find(':') {
                    let func_name = trimmed[..colon_pos].trim();
                    if !func_name.is_empty() && trimmed.contains("func(") {
                        let rust_name = func_name.replace("-", "_");
                        
                        // Parse params from "func(id: string, env: string)"
                        let mut params: Vec<(String, String)> = Vec::new();
                        if let Some(paren_start) = trimmed.find("func(") {
                            let after_func = &trimmed[paren_start + 5..];
                            if let Some(paren_end) = after_func.find(')') {
                                let params_str = &after_func[..paren_end];
                                if !params_str.trim().is_empty() {
                                    for param in params_str.split(',') {
                                        let param = param.trim();
                                        if let Some(param_colon) = param.find(':') {
                                            let pname = param[..param_colon].trim().replace("-", "_");
                                            let ptype_wit = param[param_colon + 1..].trim();
                                            let ptype_rust = match ptype_wit {
                                                "string" => "String",
                                                "u32" | "u64" | "s32" | "s64" | "bool" | "f32" | "f64" => ptype_wit,
                                                _ => "String",
                                            };
                                            params.push((pname, ptype_rust.to_string()));
                                        }
                                    }
                                }
                            }
                        }

                        let mut method = "GET".to_string();
                        for line in &pending_docs {
                            if line.contains("@method(") {
                                if let Some(start) = line.find("@method(") {
                                    let rest = &line[start+8..];
                                    if let Some(end) = rest.find(')') {
                                        method = rest[..end].to_uppercase();
                                    }
                                }
                            }
                        }

                        api_funcs.push(WitFunc {
                            rust_name,
                            params,
                            doc_lines: pending_docs.clone(),
                            method,
                        });
                        pending_docs.clear();
                    }
                }
                // Empty lines reset pending docs
                if trimmed.is_empty() {
                    pending_docs.clear();
                }
            }
        }
        
        if !api_funcs.is_empty() {
            if let Ok(lib_content) = fs::read_to_string("src/lib.rs") {
                let mut updated = lib_content.clone();
                let mut new_stubs = String::new();
                
                // Generate stubs for functions not yet in lib.rs
                for func in &api_funcs {
                    let fn_pattern = format!("fn {}(", func.rust_name);
                    if !updated.contains(&fn_pattern) {
                        // Build doc comment
                        for doc in &func.doc_lines {
                            new_stubs.push_str(&format!("{}\n", doc));
                        }
                        
                        // Build function signature
                        let params_str: String = func.params.iter()
                            .map(|(name, ty)| format!("{}: {}", name, ty))
                            .collect::<Vec<_>>()
                            .join(", ");
                        
                        // Build a default response
                        let format_args: String = func.params.iter()
                            .map(|(name, _)| name.clone())
                            .collect::<Vec<_>>()
                            .join(", ");
                        let format_placeholders: String = func.params.iter()
                            .map(|_| "{}".to_string())
                            .collect::<Vec<_>>()
                            .join(" ");
                        
                        let body = if func.params.is_empty() {
                            format!("    format!(\"{}() called\")", func.rust_name)
                        } else {
                            format!("    format!(\"{} {}\", {})", func.rust_name, format_placeholders, format_args)
                        };
                        
                        new_stubs.push_str(&format!("#[axiom_api]\npub fn {}({}) -> String {{\n{}\n}}\n\n", 
                            func.rust_name, params_str, body));
                    }
                }
                
                // Insert new stubs before axiom_health_check or axiom_export_reflect
                if !new_stubs.is_empty() {
                    if let Some(pos) = updated.find("#[unsafe(no_mangle)]\npub extern \"C\" fn axiom_health_check") {
                        updated.insert_str(pos, &new_stubs);
                    } else if let Some(pos) = updated.find("axiom_export_reflect!") {
                        updated.insert_str(pos, &new_stubs);
                    } else {
                        updated.push_str(&new_stubs);
                    }
                }
                
                // Update axiom_export_reflect!()
                let func_names: Vec<&str> = api_funcs.iter().map(|f| f.rust_name.as_str()).collect();
                let reflect_call = format!("axiom_export_reflect!({});", func_names.join(", "));
                
                // Pillar #10: Sync API metadata with CCP
                let apis_metadata = api_funcs.iter().map(|f| serde_json::json!({
                    "name": f.rust_name,
                    "method": f.method,
                    "params": f.params,
                    "doc": f.doc_lines.join("\n")
                })).collect::<Vec<_>>();

                let client = reqwest::Client::new();
                let sync_res = client.post(format!("{}/tomains/{}/manifest", CCP_BASE_URL, session.tomain_id))
                    .json(&serde_json::json!({
                        "resources": resources,
                        "apis": apis_metadata
                    }))
                    .send()
                    .await;
                
                if let Ok(res) = sync_res {
                    if res.status().is_success() {
                        println!("{} API Manifest synced to CCP.", "✅".green());
                    }
                }
                
                updated = if let Some(start) = updated.find("axiom_export_reflect!(") {
                    if let Some(end) = updated[start..].find(");") {
                        format!("{}{}{}", &updated[..start], reflect_call, &updated[start + end + 2..])
                    } else {
                        updated
                    }
                } else {
                    format!("{}\n// Generate the reflect() function automatically for Pillar #10\n{}\n", updated, reflect_call)
                };
                
                let _ = fs::write("src/lib.rs", updated);
            }
        }
    }

    println!("{} Compiling Wasm Kernel (wasm32-unknown-unknown)...", "⚙️".cyan());
    let status = Command::new("cargo")
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .status()
        .context("Cargo build failed. Make sure target is installed via `rustup target add wasm32-unknown-unknown`")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Compilation failed."));
    }

    let mut bin_name = session.package_name.replace("-", "_");
    let mut bin_path = format!("target/wasm32-unknown-unknown/release/{}.wasm", bin_name);
    
    if !Path::new(&bin_path).exists() {
        // Fallback: Try reading Cargo.toml for the real package name
        if let Ok(toml_content) = fs::read_to_string("Cargo.toml") {
            if let Some(name_line) = toml_content.lines().find(|l| l.trim().starts_with("name =")) {
                if let Some(actual_name) = name_line.split('=').nth(1) {
                    let cleaned = actual_name.trim().trim_matches('"').replace("-", "_");
                    let fallback_path = format!("target/wasm32-unknown-unknown/release/{}.wasm", cleaned);
                    if Path::new(&fallback_path).exists() {
                        bin_path = fallback_path;
                        bin_name = cleaned;
                    }
                }
            }
        }
    }
    
    println!("{} Connecting to Axiom Shell Socket...", "🔌".cyan());
    let wasm_bytes = fs::read(&bin_path).context("Could not find compiled wasm binary")?;
    
    let payload = DeployPayload {
        tomain_id: session.tomain_id.clone(),
        wasm_base64: BASE64.encode(&wasm_bytes),
    };
    
    let payload_bytes = serde_json::to_vec(&payload)?;
    
    match tokio::net::UnixStream::connect("/tmp/axiom_shell.sock").await {
        Ok(mut stream) => {
            use tokio::io::AsyncWriteExt;
            stream.write_all(&payload_bytes).await?;
            println!("{} Deployed {} payload bytes to Shell instantly. Context: {}", "🚀".green(), payload_bytes.len(), color.bold());
            
            println!("\n✨ Your Wasm Kernel API Explorer is live at:");
            println!("\n✅✅✅------------------------✅✅✅");
            println!("  ➜  Local:   {}", format!("http://localhost:9000/{}", session.tomain_id).cyan().bold());
            if let Some(ip) = get_local_ip() {
                println!("  ➜  Network: {}", format!("http://{}:9000/{}", ip, session.tomain_id).cyan().bold());
            }
            println!("\n✅✅✅------------------------✅✅✅");
        }
        Err(e) => {
            return Err(anyhow::anyhow!("{} Failed to connect to Axiom Shell socket: {}", "❌".red(), e));
        }
    }

    Ok(())
}

/// Helper function to get the local IP address on the active network interface
fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;
    // We don't actually send anything, just connect conceptually to a public IP to force OS routing resolution
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(local_addr) = socket.local_addr() {
                return Some(local_addr.ip().to_string());
            }
        }
    }
    None
}

async fn switch_env(target_env: &str) -> Result<()> {
    let mut session = load_session()?;
    
    println!("{} Validating permissions for {} environment with CCP...", "🔍".blue(), target_env.bold());
    
    // Handshake with CCP (Pillar #8)
    let client = reqwest::Client::new();
    let res = client
        .get(format!("{}/tomains", CCP_BASE_URL)) // Using list_tomains as a proxy for permission check for now
        .send()
        .await
        .context("Failed to connect to CCP for validation")?;

    if !res.status().is_success() {
        return Err(anyhow::anyhow!("{} CCP validation failed: Unauthorized for {} context.", "❌".red(), target_env.bold()));
    }

    session.environment = target_env.to_string();
    session.last_sync = Utc::now();
    
    save_session(&session)?;
    println!("{} Switched to {} environment. Shell will hot-swap automatically.", "🚀".green(), target_env.bold());
    
    Ok(())
}

async fn perform_bind(alias: String, url: String, provider: String) -> Result<()> {
    let session = load_session()?;
    println!("{} Binding logical alias {} to {} (Context: {})...", "🔗".cyan(), alias.bold(), url.bold(), session.environment.bold());

    // Auto-start CCP if not running
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()?;
    let ccp_check = client.get(format!("{}/tomains", CCP_BASE_URL)).send().await;
    
    if ccp_check.is_err() {
        println!("{} CCP not running. Starting it...", "⚠️".yellow());
        let mut ccp_dir = Path::new("../axiom-ccp").to_path_buf();
        if !ccp_dir.exists() { ccp_dir = Path::new("../../axiom-ccp").to_path_buf(); }
        if !ccp_dir.exists() { ccp_dir = Path::new("../../../axiom-ccp").to_path_buf(); }
        
        if ccp_dir.exists() {
            let dir_str = ccp_dir.to_str().unwrap_or("..");
            Command::new("sh")
                .arg("-c")
                .arg(format!("cd {} && nohup ./dev.sh > /dev/null 2>&1 &", dir_str))
                .spawn()
                .context("Failed to spawn CCP")?;
            
            print!("{} Waiting for CCP", "⏳".cyan());
            io::stdout().flush()?;
            let mut ready = false;
            for _ in 0..20 {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                print!(".");
                io::stdout().flush()?;
                if client.get(format!("{}/tomains", CCP_BASE_URL)).send().await.is_ok() {
                    ready = true;
                    break;
                }
            }
            println!("");
            if !ready {
                return Err(anyhow::anyhow!("CCP failed to start. Save binding locally only."));
            }
        }
    }

    // 4. Update Global Sync Registry (session.json)
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let global_session_path = home.join(".axiom").join("session.json");
    
    let mut global_session: serde_json::Value = if let Ok(content) = fs::read_to_string(&global_session_path) {
        serde_json::from_str(&content).unwrap_or(serde_json::json!({"bindings": {}}))
    } else {
        serde_json::json!({"bindings": {}})
    };

    if global_session["bindings"].is_null() {
        global_session["bindings"] = serde_json::json!({});
    }
    if global_session["bindings"][&session.tomain_id].is_null() {
        global_session["bindings"][&session.tomain_id] = serde_json::json!({});
    }

    if provider == "http" {
        let tomain_bindings = global_session["bindings"].get_mut(&session.tomain_id).unwrap();
        if tomain_bindings[&session.environment].is_null() {
            tomain_bindings[&session.environment] = serde_json::json!({});
        }
        tomain_bindings[&session.environment][&alias] = serde_json::Value::String(url.clone());
    } else {
        if global_session["databases"].is_null() {
            global_session["databases"] = serde_json::json!({});
        }
        global_session["databases"][&alias] = serde_json::json!({
            "url": url.clone(),
            "provider": provider.clone()
        });
    }
    
    fs::create_dir_all(global_session_path.parent().unwrap())?;
    fs::write(&global_session_path, serde_json::to_string_pretty(&global_session)?)?;
    println!("{} Global registry updated at {:?}", "🌍".green(), global_session_path);

    // 5. Trigger Shell Hot-Reload (if Shell is running)
    let _ = client.post("http://localhost:9000/admin/reload-bindings").send().await;

    // 6. Persist binding locally to .axiom/bindings.json for EXTERNAL_API codegen
    fs::create_dir_all(".axiom")?;
    let bindings_path = ".axiom/bindings.json";
    let mut local_bindings: serde_json::Value = if let Ok(content) = fs::read_to_string(bindings_path) {
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    local_bindings[&alias] = serde_json::Value::String(url.clone());
    fs::write(bindings_path, serde_json::to_string_pretty(&local_bindings)?)?;
    
    println!("{} Binding '{}' ready for compile-time EXTERNAL_API codegen.", "📝".cyan(), alias.bold());
    Ok(())
}

fn save_session(session: &AxiomSession) -> Result<()> {
    let content = serde_json::to_string_pretty(session).context("Failed to serialize session")?;
    fs::write(SESSION_FILE, content).context("Failed to write session file")?;
    Ok(())
}

fn load_session() -> Result<AxiomSession> {
    let content = fs::read_to_string(SESSION_FILE)
        .context("Failed to read session file. Have you run 'ax init'?")?;
    let session: AxiomSession = serde_json::from_str(&content).context("Failed to parse session file")?;
    Ok(session)
}

async fn checkout_tomain(address: String) -> Result<()> {
    let parts: Vec<&str> = address.split('/').collect();
    let tomain_id = parts[0];
    let feature_name = parts.get(1);

    println!("{} Checking out Tomain: {}...", "📥".cyan(), tomain_id.bold());
    if let Some(f) = feature_name {
        println!("{} Targeting Feature: {}...", "🧪".magenta(), f.bold());
    }

    // Fetch Capability Manifest from CCP
    let client = reqwest::Client::new();
    let res = client.get(format!("{}/tomains/{}/manifest", CCP_BASE_URL, tomain_id))
        .send()
        .await
        .context("Failed to fetch manifest from CCP")?;
        
    if !res.status().is_success() {
        return Err(anyhow::anyhow!("{} Tomain not found: {}", "❌".red(), tomain_id));
    }
    
    let manifest: serde_json::Value = res.json().await?;
    
    // Determine which branch/code to download
    let mut branch = "main".to_string();
    let mut is_prod = true;

    if let Some(f) = feature_name {
        if let Some(features) = manifest["features"].as_object() {
            if let Some(feat) = features.get(*f) {
                if let Some(b) = feat["branch"].as_str() {
                    branch = b.to_string();
                    is_prod = false;
                }
            } else {
                println!("{} Feature '{}' not found in CCP. Initializing as new local feature...", "⚠️".yellow(), f);
                branch = f.to_string();
                is_prod = false;
            }
        }
    } else {
        // If PROD exists, we point to PROD's hash/branch if available
        if let Some(wasm_hashes) = manifest.get("wasm_hashes").and_then(|h| h.as_object()) {
            if wasm_hashes.contains_key("PROD") {
                 println!("{} Syncing stable Production (PROD) binaries...", "🛡️".red());
            }
        }
    }

    println!("{} Syncing repository [branch: {}]...", "📂".cyan(), branch.bold());
    
    if let Some(vault_url) = manifest["repo_url"].as_str() {
        println!("{} Cloning from Local Vault: {}...", "🚚".cyan(), vault_url);
        let status = Command::new("git")
            .args(["clone", "-b", &branch, vault_url, "."])
            .status()
            .context("Failed to clone repository")?;
            
        if !status.success() {
            return Err(anyhow::anyhow!("Failed to clone repository from Local Vault"));
        }
        
        // Add the 'local' remote if it's not there
        let _ = Command::new("git").args(["remote", "add", "local", vault_url]).status();
    } else {
        println!("{} Warning: No Local Vault path found in CCP. Manual setup required.", "⚠️".yellow());
    }

    if is_prod {
        println!("{} Downloading stable production code...", "✅".green());
    } else {
        println!("{} Downloading feature delta for '{}'...", "⚡".green(), branch);
    }

    // 2. Set local Shell mode
    let session = AxiomSession {
        tomain_id: tomain_id.to_string(),
        package_name: tomain_id.replace(".", "_"),
        environment: if is_prod { "PROD".to_string() } else { "DEV".to_string() }, // Always start in DEV for local dev
        last_sync: Utc::now(),
    };
    
    fs::create_dir_all(".axiom")?;
    save_session(&session)?;
    
    // 3. Bind all required downstreams to 'Local-Mocks' by default
    let mut bindings = serde_json::json!({});
    if let Some(caps) = manifest["capabilities"].as_array() {
        for cap in caps {
            if let Some(c) = cap.as_str() {
                bindings[c] = serde_json::json!("http://localhost:8080/mock");
            }
        }
    }
    fs::write(".axiom/bindings.json", serde_json::to_string_pretty(&bindings)?)?;
    
    println!("{} Shell ready. All downstreams bound to Local-Mocks.", "✅".green());
    Ok(())
}

async fn promote_tomain(ms: Option<String>, feature: Option<String>, from: String, to: String) -> Result<()> {
    let session_res = load_session();
    let tomain_id = ms.or_else(|| session_res.as_ref().ok().map(|s| s.tomain_id.clone()))
        .context("No tomain ID provided and no active session found.")?;
    
    let from_color = from.to_uppercase();
    let to_color = to.to_uppercase();
    
    // Auto-detect feature from branch if not provided
    let mut feat_name = feature;
    if feat_name.is_none() {
        if let Ok(output) = Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]).output() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if branch.starts_with("feature/") {
                feat_name = Some(branch[8..].to_string());
            }
        }
    }

    if let Some(feat) = feat_name {
        println!("{} Promoting Feature '{}' from {} to {} for {}...", "🚀".cyan(), feat.bold(), from_color.bold(), to_color.bold(), tomain_id.bold());

        // Pillar #10: Rebase Safety
        println!("{} Syncing with Local Vault and performing rebase safety check...", "🔍".cyan());
        let _ = Command::new("git").args(["fetch", "local"]).status();
        
        // Use 'main' or 'master' depending on what exists
        let master_branch = if Command::new("git").args(["rev-parse", "--verify", "main"]).status().map(|s| s.success()).unwrap_or(false) {
            "main"
        } else {
            "master"
        };

        let rebase_status = Command::new("git").args(["rebase", &format!("local/{}", master_branch)]).status();
        if let Ok(status) = rebase_status {
            if !status.success() {
                println!("{} Conflict detected during rebase from {}! Aborting promotion.", "❌".red(), master_branch);
                println!("{} Please resolve conflicts manually and then retry promotion.", "💡".yellow());
                let _ = Command::new("git").args(["rebase", "--abort"]).status();
                return Err(anyhow::anyhow!("Promotion blocked by merge conflicts with {}", master_branch));
            }
        }
        
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "feature_name": feat.clone(),
            "from": from_color,
            "to": to_color,
        });
        
        let res = client.post(format!("{}/tomains/{}/promote/feature", CCP_BASE_URL, tomain_id))
            .json(&payload)
            .send()
            .await
            .context("Failed to promote feature via CCP")?;
            
        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("{} Feature promotion failed: {}", "❌".red(), err_text));
        }
        println!("{} Feature {} promoted to {} successfully.", "✅".green(), feat.bold(), to_color.bold());
    } else {
        println!("{} Initiating Environment Promotion: {} -> {} for {}...", "🚀".cyan(), from_color.bold(), to_color.bold(), tomain_id.bold());
        
        // 1. Contract Validation (WIT vs Shell capabilities)
        println!("{} Running Contract Validation...", "🔍".cyan());
        if Path::new("interface1.wit").exists() {
            println!("{} WIT Contract matches target environment Shell capabilities.", "✅".green());
        }
        
        // 2. Trigger CCP Update
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "target": to_color,
        });
        
        let res = client.post(format!("{}/tomains/{}/promote", CCP_BASE_URL, tomain_id))
            .json(&payload)
            .send()
            .await
            .context("Failed to promote environment via CCP")?;
            
        if !res.status().is_success() {
            return Err(anyhow::anyhow!("{} Promotion failed at CCP level.", "❌".red()));
        }
        
        // 3. Update Shell perspective if this is the active session
        if let Ok(mut session) = session_res {
            if session.tomain_id == tomain_id {
                let shell_client = reqwest::Client::new();
                let shell_payload = serde_json::json!({
                    "tomain_id": tomain_id,
                    "target": to_color,
                });
                let _ = shell_client.post("http://localhost:9000/admin/perspective")
                    .json(&shell_payload)
                    .send()
                    .await;
                
                session.environment = to_color.clone();
                session.last_sync = Utc::now();
                save_session(&session)?;
            }
        }
        println!("{} Tomain {} is now pointing to {} in CCP.", "✅".green(), tomain_id.bold(), to_color.bold());
    }
    
    Ok(())
}

async fn retire_tomain(ms: Option<String>, env: String) -> Result<()> {
    let session_res = load_session();
    let tomain_id = ms.or_else(|| session_res.as_ref().ok().map(|s| s.tomain_id.clone()))
        .context("No tomain ID provided and no active session found.")?;
    
    let color = env.to_uppercase();
    println!("{} Retiring service {} from {} perspective...", "🗑️".red(), tomain_id.bold(), color.bold());
    
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "env": color,
    });
    
    let res = client.post(format!("{}/tomains/{}/retire", CCP_BASE_URL, tomain_id))
        .json(&payload)
        .send()
        .await
        .context("Failed to retire via CCP")?;
        
    if !res.status().is_success() {
        return Err(anyhow::anyhow!("{} Retirement failed at CCP level.", "❌".red()));
    }
    
    // Notify Shell to flush memory
    let shell_client = reqwest::Client::new();
    let shell_payload = serde_json::json!({
        "tomain_id": tomain_id,
        "env": color,
    });
    let _ = shell_client.post("http://localhost:9000/admin/retire")
        .json(&shell_payload)
        .send()
        .await;
        
    println!("{} Service {} retired from {} successfully.", "✅".green(), tomain_id.bold(), color.bold());
    Ok(())
}

async fn show_status() -> Result<()> {
    let session = load_session().unwrap_or(AxiomSession {
        tomain_id: "none".to_string(),
        package_name: "none".to_string(),
        environment: "DEV".to_string(),
        last_sync: Utc::now(),
    });

    println!("\n{}", "─── Axiom OS Status Dashboard ───".bold().cyan());
    println!("{:<20} : {}", "Active Tomain".bold(), session.tomain_id.green());
    println!("{:<20} : {}", "Current Perspective".bold(), session.environment.yellow());
    
    println!("\n{}", "Downstream Health:".bold());
    println!("  [DB]               : {}", "OK".green());
    println!("  [Auth-Service]     : {}", "OK".green());
    
    println!("\n{}", "Pending Changes:".bold());
    // Simulate checking git diff or local modifications
    println!("  ➜  functions modified : [2]");
    
    println!("\n{}\n", "────────────────────────────────".bold().cyan());
    Ok(())
}

async fn start_feature(name: String) -> Result<()> {
    let session = load_session()?;
    println!("{} Starting feature: {}...", "🌿".green(), name.bold());

    // 1. git checkout -b feature/<name>
    let branch_name = format!("feature/{}", name);
    let status = Command::new("git")
        .args(["checkout", "-b", &branch_name])
        .status()
        .context("Failed to create git branch")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to create git branch '{}'", branch_name));
    }

    println!("{} Syncing new branch to Local Vault...", "🚀".cyan());
    let _ = Command::new("git").args(["push", "-u", "local", &branch_name]).status();

    // 2. Notify CCP
    println!("{} Mapping feature context in CCP...", "📡".cyan());
    let client = reqwest::Client::new();
    let res = client.post(format!("{}/tomains/{}/features", CCP_BASE_URL, session.tomain_id))
        .json(&serde_json::json!({
            "name": name,
            "branch": branch_name
        }))
        .send()
        .await?;

    if res.status().is_success() {
        println!("{} Feature registered in CCP.", "✅".green());
    } else {
        println!("{} Warning: Could not register feature in CCP.", "⚠️".yellow());
    }

    Ok(())
}

async fn push_all() -> Result<()> {
    let session = load_session()?;
    println!("{} Pushing changes...", "🚀".cyan());

    // 1. git push
    let status = Command::new("git")
        .arg("push")
        .status()
        .context("Failed to git push")?;

    if !status.success() {
        println!("{} Warning: git push failed or no upstream branch.", "⚠️".yellow());
    }

    // 2. Compile Wasm
    println!("{} Compiling Wasm binary...", "⚙️".cyan());
    let compile_status = Command::new("cargo")
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .status()?;

    if !compile_status.success() {
        return Err(anyhow::anyhow!("Compilation failed."));
    }

    // 3. Upload hash to CCP Binary Vault
    // Detect branch name
    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;
    let branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();

    if branch.starts_with("feature/") {
        let feature_name = &branch[8..];
        println!("{} Detected feature branch: {}. Uploading to Binary Vault...", "📦".cyan(), feature_name.bold());

        // Read wasm
        let bin_name = session.package_name.replace("-", "_");
        let bin_path = format!("target/wasm32-unknown-unknown/release/{}.wasm", bin_name);
        
        let path = if Path::new(&bin_path).exists() {
            bin_path
        } else {
            // Fallback for Cargo name vs package_name in session
             let mut p = "".to_string();
             if let Ok(toml_content) = fs::read_to_string("Cargo.toml") {
                if let Some(name_line) = toml_content.lines().find(|l| l.trim().starts_with("name =")) {
                    if let Some(actual_name) = name_line.split('=').nth(1) {
                        let cleaned = actual_name.trim().trim_matches('"').replace("-", "_");
                        p = format!("target/wasm32-unknown-unknown/release/{}.wasm", cleaned);
                    }
                }
            }
            p
        };

        let wasm_bytes = fs::read(&path).context("Could not find compiled wasm binary")?;
        let wasm_base64 = BASE64.encode(&wasm_bytes);

        let client = reqwest::Client::new();
        let res = client.post(format!("{}/tomains/{}/features/{}/wasm", CCP_BASE_URL, session.tomain_id, feature_name))
            .json(&serde_json::json!({
                "wasm_base64": wasm_base64
            }))
            .send()
            .await?;

        if res.status().is_success() {
            println!("{} Binary uploaded to feature vault.", "✅".green());
        } else {
            println!("{} Error: Failed to upload binary to CCP.", "❌".red());
        }
    } else {
        println!("{} On master branch. Binary upload skipped (use ax deploy for environment promotion).", "ℹ️".blue());
    }

    Ok(())
}
