// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct HttpRequest {
    method: String,
    url: String,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
    // SSL/TLS options
    verify_ssl: Option<bool>,
    ssl_cert: Option<String>,
    ssl_key: Option<String>,
    ssl_ca: Option<String>,
}

#[derive(Debug, Serialize)]
struct HttpResponse {
    status_code: u32,
    status_text: String,
    headers: HashMap<String, String>,
    body: String,
    time_ms: u64,
}

#[derive(Debug, Serialize)]
struct CurlError {
    error: String,
}

#[tauri::command]
async fn make_request(request: HttpRequest) -> Result<HttpResponse, CurlError> {
    let start_time = std::time::Instant::now();
    
    let mut easy = curl::easy::Easy::new();
    
    // Set URL
    if let Err(e) = easy.url(&request.url) {
        return Err(CurlError {
            error: format!("Invalid URL: {}", e),
        });
    }
    
    let method_upper = request.method.to_uppercase();
    let has_body = request.body.as_ref().map(|b| !b.is_empty()).unwrap_or(false);

    // Set method
    match method_upper.as_str() {
        "GET" => easy.get(true),
        "POST" => easy.post(true),
        "PUT" => {
            // Enable POST body machinery so read_function + post_field_size work
            easy.post(true);
            easy.custom_request("PUT")
        }
        "DELETE" => {
            if has_body {
                easy.post(true);
            }
            easy.custom_request("DELETE")
        }
        "PATCH" => {
            easy.post(true);
            easy.custom_request("PATCH")
        }
        "HEAD" => {
            easy.nobody(true);
            Ok(())
        }
        "OPTIONS" => {
            easy.custom_request("OPTIONS")
        }
        _ => easy.get(true),
    }.map_err(|e| CurlError {
        error: format!("Failed to set HTTP method: {}", e),
    })?;

    // Set headers
    let mut header_list = curl::easy::List::new();
    let mut has_headers = false;
    let mut has_content_type = false;
    if let Some(ref headers) = request.headers {
        for (key, value) in headers {
            if !key.to_lowercase().contains("transfer-encoding") && !key.is_empty() {
                if let Err(e) = header_list.append(&format!("{}: {}", key, value)) {
                    return Err(CurlError {
                        error: format!("Failed to add header: {}", e),
                    });
                }
                has_headers = true;
                if key.to_lowercase() == "content-type" {
                    has_content_type = true;
                }
            }
        }
    }

    // Default Content-Type for requests with a body
    if has_body && !has_content_type {
        if let Err(e) = header_list.append("Content-Type: application/json") {
            return Err(CurlError {
                error: format!("Failed to add default Content-Type header: {}", e),
            });
        }
        has_headers = true;
    }

    if has_headers {
        if let Err(e) = easy.http_headers(header_list) {
            return Err(CurlError {
                error: format!("Failed to set headers: {}", e),
            });
        }
    }
    
    // Set body
    let body_data = request.body.clone().unwrap_or_default();
    let body_for_read = Arc::new(Mutex::new(body_data.clone()));
    let body_cursor = Arc::new(Mutex::new(0usize));
    
    if !body_data.is_empty() && method_upper != "GET" {
        easy.post_field_size(body_data.len() as u64);
        
        let body_for_read_clone = Arc::clone(&body_for_read);
        let body_cursor_clone = Arc::clone(&body_cursor);
        
        easy.read_function(move |buf| {
            let body = body_for_read_clone.lock().unwrap();
            let mut cursor = body_cursor_clone.lock().unwrap();
            let remaining = &body.as_bytes()[*cursor..];
            let to_copy = std::cmp::min(buf.len(), remaining.len());
            buf[..to_copy].copy_from_slice(&remaining[..to_copy]);
            *cursor += to_copy;
            Ok(to_copy)
        }).map_err(|e| CurlError {
            error: format!("Failed to set read callback: {}", e),
        })?;
    }
    
    // Set timeouts
    easy.timeout(Duration::from_secs(30));
    easy.connect_timeout(Duration::from_secs(10));
    
    // Follow redirects
    easy.follow_location(true);
    easy.max_redirections(10);
    
    // SSL/TLS Configuration
    let verify_ssl = request.verify_ssl.unwrap_or(true);
    if !verify_ssl {
        easy.ssl_verify_peer(false);
        easy.ssl_verify_host(false);
    }
    
    // Client certificate
    if let Some(cert_path) = &request.ssl_cert {
        if !cert_path.is_empty() {
            if let Err(e) = easy.ssl_cert(cert_path) {
                return Err(CurlError {
                    error: format!("Failed to set SSL certificate: {}", e),
                });
            }
        }
    }
    
    // Client key
    if let Some(key_path) = &request.ssl_key {
        if !key_path.is_empty() {
            if let Err(e) = easy.ssl_key(key_path) {
                return Err(CurlError {
                    error: format!("Failed to set SSL key: {}", e),
                });
            }
        }
    }
    
    // CA certificate
    if let Some(ca_path) = &request.ssl_ca {
        if !ca_path.is_empty() {
            if let Err(e) = easy.cainfo(ca_path) {
                return Err(CurlError {
                    error: format!("Failed to set CA certificate: {}", e),
                });
            }
        }
    }
    
    // Buffers for response - use Arc<Mutex<>> for thread safety
    let response_headers: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let response_body: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
    
    // Header callback
    let headers_clone = Arc::clone(&response_headers);
    easy.header_function(move |data| {
        headers_clone.lock().unwrap().push(String::from_utf8_lossy(data).to_string());
        true
    }).map_err(|e| CurlError {
        error: format!("Failed to set header callback: {}", e),
    })?;
    
    // Body callback
    let body_clone = Arc::clone(&response_body);
    easy.write_function(move |data| {
        body_clone.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }).map_err(|e| CurlError {
        error: format!("Failed to set write callback: {}", e),
    })?;
    
    // Execute request
    match easy.perform() {
        Ok(_) => {
            let status_code = easy.response_code().unwrap_or(0);
            let time_ms = start_time.elapsed().as_millis() as u64;
            
            // Parse headers
            let headers = response_headers.lock().unwrap();
            let mut headers_map = HashMap::new();
            for header in headers.iter() {
                if let Some(pos) = header.find(':') {
                    let key = header[..pos].trim().to_string();
                    let value = header[pos + 1..].trim().to_string();
                    if !key.is_empty() {
                        headers_map.insert(key, value);
                    }
                }
            }
            
            let body = response_body.lock().unwrap();
            let body_string = String::from_utf8_lossy(&body).to_string();
            
            Ok(HttpResponse {
                status_code,
                status_text: get_status_text(status_code),
                headers: headers_map,
                body: body_string,
                time_ms,
            })
        }
        Err(e) => Err(CurlError {
            error: format!("Request failed: {}", e),
        }),
    }
}

fn get_status_text(code: u32) -> String {
    match code {
        100 => "Continue".to_string(),
        101 => "Switching Protocols".to_string(),
        200 => "OK".to_string(),
        201 => "Created".to_string(),
        202 => "Accepted".to_string(),
        204 => "No Content".to_string(),
        301 => "Moved Permanently".to_string(),
        302 => "Found".to_string(),
        304 => "Not Modified".to_string(),
        400 => "Bad Request".to_string(),
        401 => "Unauthorized".to_string(),
        403 => "Forbidden".to_string(),
        404 => "Not Found".to_string(),
        405 => "Method Not Allowed".to_string(),
        500 => "Internal Server Error".to_string(),
        502 => "Bad Gateway".to_string(),
        503 => "Service Unavailable".to_string(),
        504 => "Gateway Timeout".to_string(),
        _ => "Unknown".to_string(),
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![make_request])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}