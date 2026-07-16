//! Bitwarden Pop Launcher Plugin
//!
//! A plugin for the Pop!_OS / COSMIC launcher that provides access to your
//! Bitwarden vault. Passwords, usernames, and TOTP codes can be copied to the
//! clipboard. The launcher protocol lives in `plugin-common`; this file is
//! only the Bitwarden-specific logic.

use plugin_common::{
    copy_to_clipboard, send_context, send_error_result, send_finished, send_response,
    ClearResponse, CloseResponse, IconSource, PluginHandler, PluginResponse, PluginSearchResult,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::process::Command;

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

        // Bind the result before the block ends: `create_item` returns a
        // `Result<Item<'_>>` that borrows `collection`/`ss`, so evaluating
        // `.is_ok()` as the trailing expression would keep that temporary alive
        // past the locals' drop. The explicit binding drops it first.
        let stored = collection
            .create_item(
                "Bitwarden Session",
                attributes,
                session.as_bytes(),
                true, // replace if exists
                "text/plain",
            )
            .is_ok();
        stored
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

        // Arguments are passed as a vector, never through a shell. The session
        // token goes through the BW_SESSION env var (which the bw CLI reads
        // natively) instead of --session, so the secret is never exposed in
        // /proc/<pid>/cmdline to other local processes.
        let output = Command::new("bw")
            .env("BW_SESSION", &session)
            .args(["get", bw_type, &item_id])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let value = String::from_utf8_lossy(&output.stdout);
                    copy_to_clipboard(value.trim());
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!("Failed to get {}: {}", type_name, stderr);
                }
            }
            Err(e) => {
                eprintln!("Failed to run bw: {}", e);
            }
        }

        send_response(&PluginResponse::Close(CloseResponse::Close), stdout);
    }
}

impl PluginHandler for Plugin {
    fn handle_search(&mut self, query: &str, stdout: &mut io::Stdout) {
        // Clear previous results
        self.results.clear();
        send_response(&PluginResponse::Clear(ClearResponse::Clear), stdout);

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
            send_response(&PluginResponse::Append { Append: result }, stdout);
            send_finished(stdout);
            return;
        }

        // Get session
        let session = match self.get_session() {
            Some(s) => s,
            None => {
                send_error_result(
                    "Vault locked",
                    "Run 'bw unlock' and store session with 'bw-session-store'",
                    stdout,
                );
                send_finished(stdout);
                return;
            }
        };

        // Search Bitwarden vault. The query is passed as a distinct argument;
        // the session token goes via the BW_SESSION env var (not --session) so
        // it never appears in /proc/<pid>/cmdline.
        let output = Command::new("bw")
            .env("BW_SESSION", &session)
            .args(["list", "items", "--search", search_query])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    match serde_json::from_slice::<Vec<BitwardenItem>>(&output.stdout) {
                        Ok(items) => {
                            if items.is_empty() {
                                send_error_result(
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
                                    send_response(
                                        &PluginResponse::Append {
                                            Append: search_result,
                                        },
                                        stdout,
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            send_error_result("Parse error", &e.to_string(), stdout);
                        }
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.contains("Invalid session") || stderr.contains("not logged in") {
                        // Invalidate cached session
                        self.session = None;
                        send_error_result(
                            "Session expired",
                            "Run 'bw unlock' and store session again",
                            stdout,
                        );
                    } else {
                        send_error_result("Bitwarden error", &stderr, stdout);
                    }
                }
            }
            Err(e) => {
                send_error_result("Failed to run bw", &e.to_string(), stdout);
            }
        }

        send_finished(stdout);
    }

    fn handle_activate(&mut self, id: u32, stdout: &mut io::Stdout) {
        // Default action: copy password
        self.copy_credential(id, CredentialType::Password, stdout);
    }

    fn handle_context(&mut self, id: u32, stdout: &mut io::Stdout) {
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

            send_context(id, serde_json::json!(options), stdout);
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
