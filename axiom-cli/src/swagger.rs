pub fn get_swagger_html(project_name: &str) -> String {
    format!(
r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{} - Axiom API Explorer</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            background-color: #0d1117;
            color: #c9d1d9;
            margin: 0;
            padding: 40px;
        }}
        h1 {{
            color: #10B981; /* Default GREEN Theme */
        }}
        .endpoint {{
            background: #161b22;
            border-left: 5px solid #10B981;
            padding: 20px;
            margin-bottom: 15px;
            border-radius: 6px;
        }}
        .method {{
            font-weight: bold;
            color: #10B981;
            margin-right: 15px;
        }}
        .path {{
            font-family: monospace;
            font-size: 1.1em;
        }}
    </style>
</head>
<body>
    <h1>🚀 {} API Explorer</h1>
    <p>Powered by Axiom OS Anti-Gravity Wasm Runtime.</p>
    
    <h2>Available Endpoints</h2>
    
    <div class="endpoint">
        <span class="method">GET</span>
        <span class="path">/api/health</span>
        <p>Basic health check confirming the Wasm Kernel is active and responding.</p>
    </div>
</body>
</html>"#, project_name, project_name)
}
