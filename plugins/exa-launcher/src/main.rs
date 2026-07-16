//! Exa.ai Pop Launcher Plugin
//!
//! A plugin for the Pop!_OS / COSMIC launcher that provides AI-powered web
//! search via Exa.ai. The launcher protocol lives in `plugin-common`; this file
//! is only the Exa.ai-specific logic.

use plugin_common::{
    copy_to_clipboard, send_context, send_error_result, send_finished, send_response,
    truncate_string, ClearResponse, CloseResponse, IconSource, PluginHandler, PluginResponse,
    PluginSearchResult,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::process::Command;

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
}

impl PluginHandler for Plugin {
    fn handle_search(&mut self, query: &str, stdout: &mut io::Stdout) {
        // Clear previous results
        self.results.clear();
        send_response(&PluginResponse::Clear(ClearResponse::Clear), stdout);

        // Strip the "exa " prefix if present
        let search_query = query.strip_prefix("exa ").unwrap_or(query).trim();

        if search_query.is_empty() {
            send_finished(stdout);
            return;
        }

        // Check for API key
        let api_key = match &self.config.api_key {
            Some(key) => key.clone(),
            None => {
                send_error_result("Error", "No API key configured", stdout);
                send_finished(stdout);
                return;
            }
        };

        // Make API request
        let request_body = ExaSearchRequest {
            query: search_query.to_string(),
            num_results: self.config.num_results.or(Some(8)),
            contents: Some(ExaContents {
                text: ExaTextOptions {
                    max_characters: 200,
                },
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

                                send_response(
                                    &PluginResponse::Append {
                                        Append: search_result,
                                    },
                                    stdout,
                                );
                            }
                        }
                        Err(e) => {
                            send_error_result("Error", &format!("Parse error: {}", e), stdout);
                        }
                    }
                } else {
                    send_error_result("Error", &format!("API error: {}", resp.status()), stdout);
                }
            }
            Err(e) => {
                send_error_result("Error", &format!("Request failed: {}", e), stdout);
            }
        }

        send_finished(stdout);
    }

    fn handle_activate(&mut self, id: u32, stdout: &mut io::Stdout) {
        if let Some(url) = self.results.get(&id) {
            // Open URL in default browser
            let _ = Command::new("xdg-open").arg(url).spawn();
            send_response(&PluginResponse::Close(CloseResponse::Close), stdout);
        }
    }

    fn handle_context(&mut self, id: u32, stdout: &mut io::Stdout) {
        if self.results.contains_key(&id) {
            // Provide context options: Open, Copy URL
            send_context(
                id,
                serde_json::json!([
                    {"id": 0, "name": "Open in browser"},
                    {"id": 1, "name": "Copy URL to clipboard"}
                ]),
                stdout,
            );
        }
    }

    fn handle_activate_context(&mut self, id: u32, context: u32, stdout: &mut io::Stdout) {
        if let Some(url) = self.results.get(&id) {
            match context {
                0 => {
                    // Open in browser
                    let _ = Command::new("xdg-open").arg(url).spawn();
                }
                1 => {
                    // Copy to clipboard (Wayland wl-copy, X11 xclip fallback)
                    copy_to_clipboard(url);
                }
                _ => {}
            }
            send_response(&PluginResponse::Close(CloseResponse::Close), stdout);
        }
    }
}

fn main() {
    let mut plugin = Plugin::new();
    plugin.run();
}

#[cfg(test)]
mod tests {
    // The truncate_string helper and its edge cases (including multi-byte
    // characters) are now unit-tested in the shared `plugin-common` crate.
}
