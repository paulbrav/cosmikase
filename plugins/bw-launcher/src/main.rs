//! Bitwarden Pop Launcher Plugin
//!
//! A plugin for Pop!_OS launcher that provides access to your Bitwarden vault.
//! Passwords, usernames, and TOTP codes can be copied to clipboard.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};

// ============================================================================
// Pop Launcher Protocol Types
// ============================================================================

/// Requests received from pop-launcher via stdin
#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(non_snake_case, dead_code)]
enum Request {
    Activate {
        Activate: u32,
    },
    ActivateContext {
        ActivateContext: ActivateContextData,
    },
    Complete {
        Complete: u32,
    },
    Context {
        Context: u32,
    },
    Quit {
        Quit: u32,
    },
    Search {
        Search: String,
    },
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
#[allow(non_snake_case, dead_code)]
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
#[allow(dead_code)]
enum IconSource {
    Name(String),
    Mime(String),
}

// ============================================================================
// Bitwarden Types
// ============================================================================

#[derive(Debug, Deserialize, Clone)]
struct BitwardenItem {
    id: String,
    name: String,
    #[serde(default)]
    login: Option<BitwardenLogin>,
}

#[derive(Debug, Deserialize, Clone)]
struct BitwardenLogin {
    username: Option<String>,
    #[serde(default)]
    uris: Vec<BitwardenUri>,
    #[serde(default)]
    totp: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct BitwardenUri {
    uri: Option<String>,
}

// ============================================================================
// Keyring Constants
// ============================================================================

const KEYRING_SERVICE: &str = "bw-launcher";
const KEYRING_ATTRIBUTE: &str = "session";

// ============================================================================
// Plugin State
// ============================================================================

struct Plugin {
    /// Store items for activation by index
    results: HashMap<u32, BitwardenItem>,
    /// Cached session key
    session: Option<String>,
}

impl Plugin {
    fn new() -> Self {
        Plugin {
            results: HashMap::new(),
            session: None,
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
                    self.send_finished(&mut stdout);
                }
                Request::Context { Context: id } => {
                    self.handle_context(id, &mut stdout);
                }
                Request::ActivateContext {
                    ActivateContext: data,
                } => {
                    self.handle_activate_context(data.id, data.context, &mut stdout);
                }
                _ => {
                    self.send_finished(&mut stdout);
                }
            }
        }
    }

    fn get_session(&mut self) -> Option<String> {
        // Return cached session if available
        if self.session.is_some() {
            return self.session.clone();
        }

        // Try environment variable first
        if let Ok(session) = std::env::var("BW_SESSION") {
            self.session = Some(session.clone());
            return Some(session);
        }

        // Try to get session from keyring
        if let Some(session) = self.get_session_from_keyring() {
            self.session = Some(session.clone());
            return Some(session);
        }

        None
    }

    fn get_session_from_keyring(&self) -> Option<String> {
        // Use secret-service to get session from keyring
        let ss = match secret_service::blocking::SecretService::connect(
            secret_service::EncryptionType::Dh,
        ) {
            Ok(ss) => ss,
            Err(_) => return None,
        };

        let collection = match ss.get_default_collection() {
            Ok(c) => c,
            Err(_) => return None,
        };

        // Unlock collection if needed
        if collection.is_locked().unwrap_or(true) {
            let _ = collection.unlock();
        }

        let attributes = std::collections::HashMap::from([(KEYRING_ATTRIBUTE, KEYRING_SERVICE)]);
        let items = match collection.search_items(attributes) {
            Ok(items) => items,
            Err(_) => return None,
        };

        if let Some(item) = items.first() {
            if let Ok(secret) = item.get_secret() {
                return String::from_utf8(secret).ok();
            }
        }

        None
    }

    #[allow(dead_code)]
    fn store_session_in_keyring(&self, session: &str) -> bool {
        let ss = match secret_service::blocking::SecretService::connect(
            secret_service::EncryptionType::Dh,
        ) {
            Ok(ss) => ss,
            Err(_) => return false,
        };

        let collection = match ss.get_default_collection() {
            Ok(c) => c,
            Err(_) => return false,
        };

        // Unlock collection if needed
        if collection.is_locked().unwrap_or(true) {
            let _ = collection.unlock();
        }

        let attributes = std::collections::HashMap::from([(KEYRING_ATTRIBUTE, KEYRING_SERVICE)]);

        let result = collection
            .create_item(
                "Bitwarden Session",
                attributes,
                session.as_bytes(),
                true, // replace if exists
                "text/plain",
            )
            .is_ok();
        result
    }

    fn handle_search(&mut self, query: &str, stdout: &mut io::Stdout) {
        // Clear previous results
        self.results.clear();
        self.send_response(PluginResponse::Clear(ClearResponse::Clear), stdout);

        // Strip the "bw " prefix if present
        let search_query = query.strip_prefix("bw ").unwrap_or(query).trim();

        if search_query.is_empty() {
            // Show help when no query
            let result = PluginSearchResult {
                id: 0,
                name: "Bitwarden Search".to_string(),
                description: "Type to search your vault...".to_string(),
                keywords: None,
                icon: Some(IconSource::Name("bitwarden".to_string())),
                exec: None,
            };
            self.send_response(PluginResponse::Append { Append: result }, stdout);
            self.send_finished(stdout);
            return;
        }

        // Get session
        let session = match self.get_session() {
            Some(s) => s,
            None => {
                self.send_error_result(
                    "Vault locked",
                    "Run 'bw unlock' and store session with 'bw-session-store'",
                    stdout,
                );
                self.send_finished(stdout);
                return;
            }
        };

        // Search Bitwarden vault
        let output = Command::new("bw")
            .args([
                "list",
                "items",
                "--search",
                search_query,
                "--session",
                &session,
            ])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    match serde_json::from_slice::<Vec<BitwardenItem>>(&output.stdout) {
                        Ok(items) => {
                            if items.is_empty() {
                                self.send_error_result(
                                    "No results",
                                    &format!("No items found for '{}'", search_query),
                                    stdout,
                                );
                            } else {
                                for (idx, item) in items.into_iter().take(10).enumerate() {
                                    let id = idx as u32;

                                    let description = self.format_item_description(&item);

                                    let search_result = PluginSearchResult {
                                        id,
                                        name: item.name.clone(),
                                        description,
                                        keywords: None,
                                        icon: Some(IconSource::Name("dialog-password".to_string())),
                                        exec: None,
                                    };

                                    self.results.insert(id, item);
                                    self.send_response(
                                        PluginResponse::Append {
                                            Append: search_result,
                                        },
                                        stdout,
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            self.send_error_result("Parse error", &e.to_string(), stdout);
                        }
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.contains("Invalid session") || stderr.contains("not logged in") {
                        // Invalidate cached session
                        self.session = None;
                        self.send_error_result(
                            "Session expired",
                            "Run 'bw unlock' and store session again",
                            stdout,
                        );
                    } else {
                        self.send_error_result("Bitwarden error", &stderr, stdout);
                    }
                }
            }
            Err(e) => {
                self.send_error_result("Failed to run bw", &e.to_string(), stdout);
            }
        }

        self.send_finished(stdout);
    }

    fn format_item_description(&self, item: &BitwardenItem) -> String {
        let mut parts = Vec::new();

        if let Some(ref login) = item.login {
            if let Some(ref username) = login.username {
                parts.push(username.clone());
            }
            if let Some(uri) = login.uris.first().and_then(|u| u.uri.as_ref()) {
                // Extract domain from URI
                let domain = uri
                    .trim_start_matches("https://")
                    .trim_start_matches("http://")
                    .split('/')
                    .next()
                    .unwrap_or(uri);
                parts.push(domain.to_string());
            }
        }

        if parts.is_empty() {
            "Login item".to_string()
        } else {
            parts.join(" • ")
        }
    }

    fn handle_activate(&mut self, id: u32, stdout: &mut io::Stdout) {
        // Default action: copy password
        self.copy_credential(id, CredentialType::Password, stdout);
    }

    fn handle_context(&self, id: u32, stdout: &mut io::Stdout) {
        if let Some(item) = self.results.get(&id) {
            let mut options = vec![
                serde_json::json!({"id": 0, "name": "Copy password"}),
                serde_json::json!({"id": 1, "name": "Copy username"}),
            ];

            // Add TOTP option if available
            if item
                .login
                .as_ref()
                .map(|l| l.totp.is_some())
                .unwrap_or(false)
            {
                options.push(serde_json::json!({"id": 2, "name": "Copy TOTP code"}));
            }

            let context_response = serde_json::json!({
                "Context": {
                    "id": id,
                    "options": options
                }
            });
            let _ = writeln!(stdout, "{}", context_response);
            let _ = stdout.flush();
        }
    }

    fn handle_activate_context(&mut self, id: u32, context: u32, stdout: &mut io::Stdout) {
        let credential_type = match context {
            0 => CredentialType::Password,
            1 => CredentialType::Username,
            2 => CredentialType::Totp,
            _ => return,
        };

        self.copy_credential(id, credential_type, stdout);
    }

    fn copy_credential(
        &mut self,
        id: u32,
        credential_type: CredentialType,
        stdout: &mut io::Stdout,
    ) {
        let item_id = match self.results.get(&id) {
            Some(item) => item.id.clone(),
            None => return,
        };

        let session = match self.get_session() {
            Some(s) => s,
            None => {
                return;
            }
        };

        let (bw_type, type_name) = match credential_type {
            CredentialType::Password => ("password", "Password"),
            CredentialType::Username => ("username", "Username"),
            CredentialType::Totp => ("totp", "TOTP"),
        };

        let output = Command::new("bw")
            .args(["get", bw_type, &item_id, "--session", &session])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let value = String::from_utf8_lossy(&output.stdout);
                    self.copy_to_clipboard(value.trim());
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!("Failed to get {}: {}", type_name, stderr);
                }
            }
            Err(e) => {
                eprintln!("Failed to run bw: {}", e);
            }
        }

        self.send_response(PluginResponse::Close(CloseResponse::Close), stdout);
    }

    fn copy_to_clipboard(&self, text: &str) {
        // Try wl-copy first (Wayland)
        if Command::new("wl-copy").arg(text).status().is_ok() {
            return;
        }

        // Fallback to xclip (X11)
        if let Ok(mut child) = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
        }
    }

    fn send_error_result(&self, title: &str, message: &str, stdout: &mut io::Stdout) {
        let result = PluginSearchResult {
            id: 999,
            name: title.to_string(),
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

#[derive(Debug, Clone, Copy)]
enum CredentialType {
    Password,
    Username,
    Totp,
}

fn main() {
    let mut plugin = Plugin::new();
    plugin.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_item_description() {
        let plugin = Plugin::new();
        let item = BitwardenItem {
            id: "123".to_string(),
            name: "Test".to_string(),
            login: Some(BitwardenLogin {
                username: Some("user".to_string()),
                uris: vec![BitwardenUri {
                    uri: Some("https://example.com/path".to_string()),
                }],
                totp: None,
            }),
        };
        assert_eq!(plugin.format_item_description(&item), "user • example.com");
    }

    #[test]
    fn test_format_item_description_no_login() {
        let plugin = Plugin::new();
        let item = BitwardenItem {
            id: "123".to_string(),
            name: "Test".to_string(),
            login: None,
        };
        assert_eq!(plugin.format_item_description(&item), "Login item");
    }
}
