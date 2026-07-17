//! Bitwarden Pop Launcher Plugin
//!
//! A plugin for the Pop!_OS / COSMIC launcher that provides access to your
//! Bitwarden vault. Passwords, usernames, and TOTP codes can be copied to the
//! clipboard. The launcher protocol lives in `plugin-common`; this file is
//! only the Bitwarden-specific logic.

use plugin_common::{
    copy_to_clipboard, Activation, ContextOption, IconSource, PluginHandler, Row, Search,
};
use serde::Deserialize;
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
    /// Cached session key
    session: Option<String>,
}

impl Plugin {
    fn new() -> Self {
        Plugin { session: None }
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

    /// Copy the requested credential of `item` to the clipboard. Returns
    /// `KeepOpen` (deliberately withholding the launcher `Close`) when no
    /// session is available, so the user sees the vault is locked instead of the
    /// launcher silently closing.
    fn copy_credential(
        &mut self,
        item: &BitwardenItem,
        credential_type: CredentialType,
    ) -> Activation {
        let session = match self.get_session() {
            Some(s) => s,
            None => return Activation::KeepOpen,
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
            .args(["get", bw_type, &item.id])
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

        Activation::Close
    }
}

impl PluginHandler for Plugin {
    type Item = BitwardenItem;
    const PREFIX: &'static str = "bw ";

    fn search(&mut self, query: &str) -> Search<BitwardenItem> {
        if query.is_empty() {
            // Show help when no query
            return Search::help(
                "Bitwarden Search",
                "Type to search your vault...",
                IconSource::Name("bitwarden".to_string()),
            );
        }

        // Get session
        let session = match self.get_session() {
            Some(s) => s,
            None => {
                return Search::error(
                    "Vault locked",
                    "Store a session: secret-tool store --label='Bitwarden Session' session bw-launcher, or export BW_SESSION=$(bw unlock --raw)",
                )
            }
        };

        // Search Bitwarden vault. The query is passed as a distinct argument;
        // the session token goes via the BW_SESSION env var (not --session) so
        // it never appears in /proc/<pid>/cmdline.
        let output = Command::new("bw")
            .env("BW_SESSION", &session)
            .args(["list", "items", "--search", query])
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    match serde_json::from_slice::<Vec<BitwardenItem>>(&output.stdout) {
                        Ok(items) => {
                            if items.is_empty() {
                                Search::error(
                                    "No results",
                                    format!("No items found for '{}'", query),
                                )
                            } else {
                                Search::Results(items.into_iter().take(10).collect())
                            }
                        }
                        Err(e) => Search::error("Parse error", e.to_string()),
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.contains("Invalid session") || stderr.contains("not logged in") {
                        // Invalidate cached session
                        self.session = None;
                        Search::error("Session expired", "Run 'bw unlock' and store session again")
                    } else {
                        Search::error("Bitwarden error", stderr.into_owned())
                    }
                }
            }
            Err(e) => Search::error("Failed to run bw", e.to_string()),
        }
    }

    fn row(&self, item: &BitwardenItem) -> Row {
        Row::new(
            item.name.clone(),
            self.format_item_description(item),
            IconSource::Name("dialog-password".to_string()),
        )
    }

    fn activate(&mut self, item: &BitwardenItem) -> Activation {
        // Default action: copy password
        self.copy_credential(item, CredentialType::Password)
    }

    fn context_menu(&self, item: &BitwardenItem) -> Vec<ContextOption> {
        let mut options = vec![
            ContextOption::new(0, "Copy password"),
            ContextOption::new(1, "Copy username"),
        ];

        // Add TOTP option if available
        if item
            .login
            .as_ref()
            .map(|l| l.totp.is_some())
            .unwrap_or(false)
        {
            options.push(ContextOption::new(2, "Copy TOTP code"));
        }

        options
    }

    fn activate_context(&mut self, item: &BitwardenItem, context: u32) -> Activation {
        let credential_type = match context {
            0 => CredentialType::Password,
            1 => CredentialType::Username,
            2 => CredentialType::Totp,
            _ => return Activation::KeepOpen,
        };

        self.copy_credential(item, credential_type)
    }
}

#[derive(Debug, Clone, Copy)]
enum CredentialType {
    Password,
    Username,
    Totp,
}

fn main() {
    Plugin::new().run();
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
