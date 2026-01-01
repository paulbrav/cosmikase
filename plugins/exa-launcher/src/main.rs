//! Exa.ai Pop Launcher Plugin
//!
//! A plugin for Pop!_OS launcher that provides AI-powered web search via Exa.ai.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::Command;

// ============================================================================
// Pop Launcher Protocol Types
// ============================================================================

/// Requests received from pop-launcher via stdin
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Request {
    Activate { Activate: u32 },
    ActivateContext { ActivateContext: ActivateContextData },
    Complete { Complete: u32 },
    Context { Context: u32 },
    Quit { Quit: u32 },
    Search { Search: String },
    Simple(SimpleRequest),
}

#[derive(Debug, Deserialize)]
struct ActivateContextData {
    id: u32,
    context: u32,
}

#[derive(Debug, Deserialize)]
enum SimpleRequest {
    Exit,
    Interrupt,
}

/// Responses sent to pop-launcher via stdout
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum PluginResponse {
    Append { Append: PluginSearchResult },
    Clear(ClearResponse),
    Close(CloseResponse),
    Fill { Fill: String },
    Finished(FinishedResponse),
}

#[derive(Debug, Serialize)]
enum ClearResponse {
    Clear,
}

#[derive(Debug, Serialize)]
enum CloseResponse {
    Close,
}

#[derive(Debug, Serialize)]
enum FinishedResponse {
    Finished,
}

#[derive(Debug, Serialize)]
struct PluginSearchResult {
    id: u32,
    name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    keywords: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<IconSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exec: Option<String>,
}

#[derive(Debug, Serialize)]
enum IconSource {
    Name(String),
    Mime(String),
}

// ============================================================================
// Exa.ai API Types
// ============================================================================

#[derive(Debug, Serialize)]
struct ExaSearchRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_results: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contents: Option<ExaContents>,
}

#[derive(Debug, Serialize)]
struct ExaContents {
    text: ExaTextOptions,
}

#[derive(Debug, Serialize)]
struct ExaTextOptions {
    max_characters: u32,
}

#[derive(Debug, Deserialize)]
struct ExaSearchResponse {
    results: Vec<ExaResult>,
}

#[derive(Debug, Deserialize)]
struct ExaResult {
    title: Option<String>,
    url: String,
    #[serde(default)]
    text: Option<String>,
}

// ============================================================================
// Configuration
// ============================================================================

#[derive(Debug, Deserialize, Default)]
struct Config {
    api_key: Option<String>,
    num_results: Option<u32>,
}

impl Config {
    fn load() -> Self {
        // Try environment variable first
        if let Ok(api_key) = std::env::var("EXA_API_KEY") {
            return Config {
                api_key: Some(api_key),
                num_results: Some(8),
            };
        }

        // Try config file
        let config_path = Self::config_path();
        if let Ok(contents) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str::<Config>(&contents) {
                return config;
            }
        }

        Config::default()
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("exa-launcher")
            .join("config.toml")
    }
}

// ============================================================================
// Plugin State
// ============================================================================

struct Plugin {
    config: Config,
    client: reqwest::blocking::Client,
    /// Store URLs for activation by index
    results: HashMap<u32, String>,
}

impl Plugin {
    fn new() -> Self {
        let config = Config::load();
        let client = reqwest::blocking::Client::new();
        Plugin {
            config,
            client,
            results: HashMap::new(),
        }
    }

    fn run(&mut self) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };

            if line.is_empty() {
                continue;
            }

            let request: Request = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(_) => continue,
            };

            match request {
                Request::Search { Search: query } => {
                    self.handle_search(&query, &mut stdout);
                }
                Request::Activate { Activate: id } => {
                    self.handle_activate(id, &mut stdout);
                }
                Request::Simple(SimpleRequest::Exit) => {
                    break;
                }
                Request::Simple(SimpleRequest::Interrupt) => {
                    // Cancel current operation - just send Finished
                    self.send_finished(&mut stdout);
                }
                Request::Context { Context: id } => {
                    self.handle_context(id, &mut stdout);
                }
                Request::ActivateContext { ActivateContext: data } => {
                    self.handle_activate_context(data.id, data.context, &mut stdout);
                }
                _ => {
                    self.send_finished(&mut stdout);
                }
            }
        }
    }

    fn handle_search(&mut self, query: &str, stdout: &mut io::Stdout) {
        // Clear previous results
        self.results.clear();
        self.send_response(PluginResponse::Clear(ClearResponse::Clear), stdout);

        // Strip the "exa " prefix if present
        let search_query = query.strip_prefix("exa ").unwrap_or(query).trim();

        if search_query.is_empty() {
            self.send_finished(stdout);
            return;
        }

        // Check for API key
        let api_key = match &self.config.api_key {
            Some(key) => key.clone(),
            None => {
                self.send_error_result("No API key configured", stdout);
                self.send_finished(stdout);
                return;
            }
        };

        // Make API request
        let request_body = ExaSearchRequest {
            query: search_query.to_string(),
            num_results: self.config.num_results.or(Some(8)),
            contents: Some(ExaContents {
                text: ExaTextOptions { max_characters: 200 },
            }),
        };

        let response = self
            .client
            .post("https://api.exa.ai/search")
            .header("x-api-key", &api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send();

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<ExaSearchResponse>() {
                        Ok(exa_response) => {
                            for (idx, result) in exa_response.results.into_iter().enumerate() {
                                let id = idx as u32;
                                let title = result.title.unwrap_or_else(|| "Untitled".to_string());
                                let description = result
                                    .text
                                    .map(|t| truncate_string(&t, 100))
                                    .unwrap_or_else(|| result.url.clone());

                                // Store URL for activation
                                self.results.insert(id, result.url);

                                let search_result = PluginSearchResult {
                                    id,
                                    name: title,
                                    description,
                                    keywords: None,
                                    icon: Some(IconSource::Name("web-browser".to_string())),
                                    exec: None,
                                };

                                self.send_response(
                                    PluginResponse::Append {
                                        Append: search_result,
                                    },
                                    stdout,
                                );
                            }
                        }
                        Err(e) => {
                            self.send_error_result(&format!("Parse error: {}", e), stdout);
                        }
                    }
                } else {
                    self.send_error_result(
                        &format!("API error: {}", resp.status()),
                        stdout,
                    );
                }
            }
            Err(e) => {
                self.send_error_result(&format!("Request failed: {}", e), stdout);
            }
        }

        self.send_finished(stdout);
    }

    fn handle_activate(&self, id: u32, stdout: &mut io::Stdout) {
        if let Some(url) = self.results.get(&id) {
            // Open URL in default browser
            let _ = Command::new("xdg-open").arg(url).spawn();
            self.send_response(PluginResponse::Close(CloseResponse::Close), stdout);
        }
    }

    fn handle_context(&self, id: u32, stdout: &mut io::Stdout) {
        if self.results.contains_key(&id) {
            // Provide context options: Open, Copy URL
            let context_response = serde_json::json!({
                "Context": {
                    "id": id,
                    "options": [
                        {"id": 0, "name": "Open in browser"},
                        {"id": 1, "name": "Copy URL to clipboard"}
                    ]
                }
            });
            let _ = writeln!(stdout, "{}", context_response);
            let _ = stdout.flush();
        }
    }

    fn handle_activate_context(&self, id: u32, context: u32, stdout: &mut io::Stdout) {
        if let Some(url) = self.results.get(&id) {
            match context {
                0 => {
                    // Open in browser
                    let _ = Command::new("xdg-open").arg(url).spawn();
                }
                1 => {
                    // Copy to clipboard using wl-copy (Wayland) or xclip (X11)
                    if Command::new("wl-copy").arg(url).status().is_err() {
                        let _ = Command::new("xclip")
                            .args(["-selection", "clipboard"])
                            .stdin(std::process::Stdio::piped())
                            .spawn()
                            .and_then(|mut child| {
                                if let Some(stdin) = child.stdin.as_mut() {
                                    stdin.write_all(url.as_bytes())?;
                                }
                                child.wait()
                            });
                    }
                }
                _ => {}
            }
            self.send_response(PluginResponse::Close(CloseResponse::Close), stdout);
        }
    }

    fn send_error_result(&self, message: &str, stdout: &mut io::Stdout) {
        let result = PluginSearchResult {
            id: 999,
            name: "Error".to_string(),
            description: message.to_string(),
            keywords: None,
            icon: Some(IconSource::Name("dialog-error".to_string())),
            exec: None,
        };
        self.send_response(PluginResponse::Append { Append: result }, stdout);
    }

    fn send_finished(&self, stdout: &mut io::Stdout) {
        self.send_response(PluginResponse::Finished(FinishedResponse::Finished), stdout);
    }

    fn send_response(&self, response: PluginResponse, stdout: &mut io::Stdout) {
        if let Ok(json) = serde_json::to_string(&response) {
            let _ = writeln!(stdout, "{}", json);
            let _ = stdout.flush();
        }
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn main() {
    let mut plugin = Plugin::new();
    plugin.run();
}





