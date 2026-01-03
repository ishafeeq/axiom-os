use axum::{
    extract::{Path, State},
    response::Html,
};
use tracing::info;

use crate::handlers::registry::AppState;

pub async fn get_swagger_ui(
    State(_state): State<AppState>,
    Path(package_id): Path<String>,
) -> Html<String> {
    info!("Requesting API Manifest docs for Tomain: {}", package_id);
    
    let shell_url = format!("http://localhost:9000/reflect/{}", package_id);
    let client = reqwest::Client::new();
    
    match client.get(&shell_url).send().await {
        Ok(res) => {
            let status = res.status();
            if status.is_success() {
                if let Ok(json_spec) = res.text().await {
                    let friendly_name = package_id.split('.').last().unwrap_or(&package_id);
                    let dynamic_spec = json_spec.replace("\"Axiom Kernel API\"", &format!("\"{} API\"", friendly_name));
                    return Html(render_swagger_template(&dynamic_spec, friendly_name));
                }
            }
            Html(render_error_template(&package_id, &format!("Shell returned status {}", status)))
        },
        Err(_) => {
            Html(render_error_template(&package_id, "Could not connect to Axiom Shell runtime."))
        }
    }
}

fn render_error_template(package_id: &str, reason: &str) -> String {
    format!(
r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>API Explorer Error - {}</title>
    <style>
        body {{ 
            margin: 0; 
            background: #0d1117; 
            color: #c9d1d9; 
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100vh;
            text-align: center;
        }}
        .container {{
            max-width: 600px;
            padding: 40px;
            background: #161b22;
            border: 1px border #30363d;
            border-radius: 12px;
            box-shadow: 0 8px 24px rgba(0,0,0,0.5);
        }}
        h1 {{ color: #ff7b72; font-size: 24px; margin-bottom: 16px; }}
        p {{ color: #8b949e; line-height: 1.6; margin-bottom: 24px; }}
        .code-box {{
            background: #0d1117;
            padding: 12px;
            border-radius: 6px;
            font-family: ui-monospace, SFMono-Regular, SF Mono, Menlo, Consolas, Liberation Mono, monospace;
            font-size: 14px;
            color: #79c0ff;
            margin-bottom: 24px;
            border: 1px solid #30363d;
        }}
        .hint {{
            color: #d29922;
            font-weight: bold;
            font-size: 14px;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1 style="display: flex; align-items: center; justify-content: center; gap: 8px;">
            <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="m19 5 3-3"></path>
                <path d="m2 22 3-3"></path>
                <path d="M6.3 20.3a2.4 2.4 0 0 0 3.4 0L12 18l-6-6-2.3 2.3a2.4 2.4 0 0 0 0 3.4Z"></path>
                <path d="M7.5 13.5 10 16"></path>
                <path d="M10.5 16.5 13 19"></path>
                <path d="m12 6 6 6 2.3-2.3a2.4 2.4 0 0 0 0-3.4l-3.1-3.1a2.4 2.4 0 0 0-3.4 0Z"></path>
            </svg>
            Service is down
        </h1>
        <p>{}</p>
        <div class="hint">Troubleshooting</div>
        <p>Please check if the Axiom Shell is running for this package. Try executing:</p>
        <div class="code-box">ax deploy dev</div>
        <p style="font-size: 13px;">This will ensure the kernel is compiled and loaded into a live Shell slot.</p>
    </div>
</body>
</html>"#, package_id, reason)
}

fn render_swagger_template(json_spec: &str, title: &str) -> String {
    format!(
r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{} - Axiom API Explorer</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" >
    <style>
        html {{ box-sizing: border-box; overflow-y: scroll; }}
        body {{ margin:0; background: #0d1117; color: #c9d1d9; }}
        /* Dark Mode overrides for Swagger UI */
        .swagger-ui {{ filter: invert(88%) hue-rotate(180deg); }}
        .swagger-ui .topbar {{ display: none; }}
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"> </script>
    <script>
    window.onload = function() {{
      window.ui = SwaggerUIBundle({{
        spec: {},
        dom_id: '#swagger-ui',
        deepLinking: true,
        presets: [
          SwaggerUIBundle.presets.apis
        ],
      }});
    }};
    </script>
</body>
</html>"#, title, json_spec)
}
