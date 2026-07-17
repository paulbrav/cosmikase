//! Exa.ai Pop Launcher Plugin
//!
//! A plugin for the Pop!_OS / COSMIC launcher that provides AI-powered web
//! search via Exa.ai. The launcher protocol lives in `plugin-common`; this file
//! is only the Exa.ai-specific logic.

use plugin_common::{
    copy_to_clipboard, truncate_string, Activation, ContextOption, IconSource, PluginHandler, Row,
    Search,
};
use serde::{Deserialize, Serialize};
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
}

impl Plugin {
    fn new() -> Self {
        Plugin {
            config: Config::load(),
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl PluginHandler for Plugin {
    type Item = ExaResult;
    const PREFIX: &'static str = "exa ";

    fn search(&mut self, query: &str) -> Search<ExaResult> {
        if query.is_empty() {
            // Nothing to show for an empty query.
            return Search::Results(Vec::new());
        }

        // Check for API key
        let api_key = match &self.config.api_key {
            Some(key) => key.clone(),
            None => return Search::error("Error", "No API key configured"),
        };

        // Make API request
        let request_body = ExaSearchRequest {
            query: query.to_string(),
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
                        Ok(exa_response) => Search::Results(exa_response.results),
                        Err(e) => Search::error("Error", format!("Parse error: {}", e)),
                    }
                } else {
                    Search::error("Error", format!("API error: {}", resp.status()))
                }
            }
            Err(e) => Search::error("Error", format!("Request failed: {}", e)),
        }
    }

    fn row(&self, result: &ExaResult) -> Row {
        let name = result
            .title
            .clone()
            .unwrap_or_else(|| "Untitled".to_string());
        let description = result
            .text
            .as_ref()
            .map(|t| truncate_string(t, 100))
            .unwrap_or_else(|| result.url.clone());
        Row::new(
            name,
            description,
            IconSource::Name("web-browser".to_string()),
        )
    }

    fn activate(&mut self, result: &ExaResult) -> Activation {
        // Open URL in default browser
        let _ = Command::new("xdg-open").arg(&result.url).spawn();
        Activation::Close
    }

    fn context_menu(&self, _result: &ExaResult) -> Vec<ContextOption> {
        vec![
            ContextOption::new(0, "Open in browser"),
            ContextOption::new(1, "Copy URL to clipboard"),
        ]
    }

    fn activate_context(&mut self, result: &ExaResult, context: u32) -> Activation {
        match context {
            0 => {
                let _ = Command::new("xdg-open").arg(&result.url).spawn();
            }
            1 => copy_to_clipboard(&result.url),
            _ => {}
        }
        Activation::Close
    }
}

fn main() {
    Plugin::new().run();
}

#[cfg(test)]
mod tests {
    // The truncate_string helper and its edge cases (including multi-byte
    // characters) are now unit-tested in the shared `plugin-common` crate.
}
