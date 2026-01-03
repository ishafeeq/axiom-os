extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Attribute, Lit, Meta, ReturnType, FnArg, Pat};

#[proc_macro_attribute]
pub fn axiom_api(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let vis = &input.vis;
    let attrs = &input.attrs;
    let sig = &input.sig;
    let block = &input.block;

    // Extract doc comments
    let mut docs = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(nv) = &attr.meta {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
                    docs.push(s.value().trim().to_string());
                }
            }
        }
    }
    let summary = docs.join("\n");

    // Extract params
    let mut params = Vec::new();
    for arg in &sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let name = pat_ident.ident.to_string();
                // For simplicity, we just store the name. 
                // In a full impl, we'd map Rust types to OpenAPI types.
                params.push(name);
            }
        }
    }

    // Generic invocation wrapper
    let invoke_fn_name = quote::format_ident!("__axiom_call_{}", fn_name);
    let metadata_fn_name = quote::format_ident!("__axiom_metadata_{}", fn_name);
    let params_tokens = params.iter().map(|p| quote! { #p });

    let args_count = sig.inputs.len();

    // Generate code to extract each param by name from the JSON
    let param_extractions = params.iter().map(|p| {
        let param_name = p.clone();
        let param_ident = quote::format_ident!("arg_{}", p);
        quote! {
            let #param_ident = args_json[#param_name].as_str().unwrap_or("").to_string();
        }
    });

    let param_idents = params.iter().map(|p| {
        let param_ident = quote::format_ident!("arg_{}", p);
        quote! { #param_ident }
    });

    let expanded = quote! {
        #input

        #[unsafe(no_mangle)]
        pub extern "C" fn #invoke_fn_name(json_ptr: u32, json_len: u32) -> *const u8 {
            // Read the JSON string from the pointer provided by the Shell host
            let json_str = if json_ptr > 0 && json_len > 0 {
                let slice = unsafe { core::slice::from_raw_parts(json_ptr as *const u8, json_len as usize) };
                core::str::from_utf8(slice).unwrap_or("{}")
            } else {
                "{}"
            };

            let args_json: serde_json::Value = serde_json::from_str(json_str).unwrap_or(serde_json::json!({}));

            #(#param_extractions)*

            let res = #fn_name(#(#param_idents),*);
            let res_with_null = format!("{}\0", res);
            let s = Box::leak(res_with_null.into_boxed_str());
            s.as_ptr()
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn #metadata_fn_name() -> *const u8 {
            let json = serde_json::json!({
                "name": stringify!(#fn_name),
                "summary": #summary,
                "parameters": [#(#params_tokens),*],
                "invoke": stringify!(#invoke_fn_name)
            }).to_string();
            let json_with_null = format!("{}\0", json);
            let s = Box::leak(json_with_null.into_boxed_str());
            s.as_ptr()
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro]
pub fn axiom_export_reflect(input: TokenStream) -> TokenStream {
    let idents = syn::parse_macro_input!(input with syn::punctuated::Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated);
    
    let metadata_calls = idents.iter().map(|ident| {
        let metadata_fn_name = quote::format_ident!("__axiom_metadata_{}", ident);
        quote! {
            let meta_ptr = #metadata_fn_name();
            let meta_str = unsafe { std::ffi::CStr::from_ptr(meta_ptr as *const i8).to_str().unwrap_or("{}") };
            let meta_json: serde_json::Value = serde_json::from_str(meta_str).unwrap_or(serde_json::json!({}));
            
            let name = meta_json["name"].as_str().unwrap_or("unknown");
            let summary = meta_json["summary"].as_str().unwrap_or("");
            let params = meta_json["parameters"].as_array().cloned().unwrap_or_default();
            
            // Map to OpenAPI path
            let path = format!("/{}", name.replace("_", "-"));
            let method = if summary.to_lowercase().contains("delete") || name.contains("delete") { 
                "delete" 
            } else if summary.to_lowercase().contains("put") || name.contains("put") { 
                "put" 
            } else if summary.to_lowercase().contains("post") || name.contains("post") || name.contains("submit") { 
                "post" 
            } else { 
                "get" 
            };
            
            // For GET/DELETE: use query params. For POST/PUT: use requestBody
            let endpoint_spec = if method == "get" || method == "delete" {
                serde_json::json!({
                    "summary": summary,
                    "parameters": params.iter().map(|p| {
                        serde_json::json!({
                            "name": p,
                            "in": "query",
                            "required": true,
                            "schema": { "type": "string" }
                        })
                    }).collect::<Vec<_>>()
                })
            } else {
                // Build a JSON Schema from param names
                let mut properties = serde_json::Map::new();
                for p in params.iter() {
                    properties.insert(
                        p.as_str().unwrap_or("unknown").to_string(),
                        serde_json::json!({ "type": "string" })
                    );
                }
                let required: Vec<_> = params.iter()
                    .filter_map(|p| p.as_str().map(|s| s.to_string()))
                    .collect();
                    
                serde_json::json!({
                    "summary": summary,
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": properties,
                                    "required": required
                                }
                            }
                        }
                    }
                })
            };
            
            paths.insert(path, serde_json::json!({
                method: endpoint_spec
            }));
        }
    });

    let expanded = quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn reflect() -> *const u8 {
            let mut paths = std::collections::HashMap::new();
            
            // Add health check by default
            paths.insert("/health".to_string(), serde_json::json!({
                "get": { "summary": "Health Check" }
            }));

            #( #metadata_calls )*

            let manifest = serde_json::json!({
                "openapi": "3.0.0",
                "info": { "title": "Axiom Kernel API", "version": "1.0.0" },
                "servers": [
                    { "url": "http://localhost:9000", "description": "Local Axiom Shell" }
                ],
                "paths": paths
            });

            let json_str = format!("{}\0", manifest.to_string());
            let s = Box::leak(json_str.into_boxed_str());
            s.as_ptr()
        }
    };
    TokenStream::from(expanded)
}
