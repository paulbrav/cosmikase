//! SSH Hosts Pop Launcher Plugin
//!
//! A plugin for the Pop!_OS / COSMIC launcher that provides quick access to SSH
//! hosts defined in ~/.ssh/config. The launcher protocol lives in
//! `plugin-common`; this file is only the SSH-config-specific logic.

use plugin_common::{
    copy_to_clipboard, spawn_in_terminal, Activation, ContextOption, IconSource, PluginHandler,
    Row, Search,
};
use std::fs;
use std::path::PathBuf;

// ============================================================================
// SSH Config Types
// ============================================================================

#[derive(Debug, Clone)]
struct SshHost {
    name: String,
    hostname: Option<String>,
    user: Option<String>,
    port: Option<u16>,
}

impl SshHost {
    fn description(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref user) = self.user {
            parts.push(format!("{}@", user));
        }

        if let Some(ref hostname) = self.hostname {
            parts.push(hostname.clone());
        } else {
            parts.push(self.name.clone());
        }

        if let Some(port) = self.port {
            if port != 22 {
                parts.push(format!(":{}", port));
            }
        }

        parts.join("")
    }

    fn ssh_command(&self) -> String {
        format!("ssh {}", self.name)
    }

    /// Open an SSH session to this host inside a terminal. The host alias is
    /// passed as a distinct argv entry (`ssh <name>`) — no shell, so an alias
    /// containing shell metacharacters cannot be interpreted as a command.
    fn open_in_terminal(&self) {
        spawn_in_terminal(&["ssh", &self.name]);
    }

    fn matches(&self, query: &str) -> bool {
        self.name.to_lowercase().contains(query)
            || self
                .hostname
                .as_ref()
                .map(|hn| hn.to_lowercase().contains(query))
                .unwrap_or(false)
            || self
                .user
                .as_ref()
                .map(|u| u.to_lowercase().contains(query))
                .unwrap_or(false)
    }
}

// ============================================================================
// Plugin State
// ============================================================================

struct Plugin {
    /// Cached hosts from SSH config
    hosts: Vec<SshHost>,
}

impl Plugin {
    fn new() -> Self {
        Plugin {
            hosts: Self::parse_ssh_config(),
        }
    }

    fn parse_ssh_config() -> Vec<SshHost> {
        let config_path = Self::ssh_config_path();
        let contents = match fs::read_to_string(&config_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let mut hosts = Vec::new();
        let mut current_host: Option<SshHost> = None;

        // ssh_config lines are `Keyword value...`; the keyword is
        // case-insensitive. Split on the first run of whitespace and dispatch on
        // the lowercased keyword — no regex needed.
        for line in contents.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (keyword, value) = match line.split_once(char::is_whitespace) {
                Some((k, v)) => (k.to_lowercase(), v.trim()),
                None => continue,
            };

            match keyword.as_str() {
                "host" => {
                    // Save previous host if any
                    if let Some(host) = current_host.take() {
                        // Filter out wildcard patterns
                        if !host.name.contains('*') && !host.name.contains('?') {
                            hosts.push(host);
                        }
                    }

                    // Parse host names (can be multiple space-separated)
                    for host_name in value.split_whitespace() {
                        // Skip wildcards
                        if host_name.contains('*') || host_name.contains('?') {
                            continue;
                        }
                        current_host = Some(SshHost {
                            name: host_name.to_string(),
                            hostname: None,
                            user: None,
                            port: None,
                        });
                        break; // Only take the first non-wildcard host
                    }
                }
                "hostname" => {
                    if let Some(ref mut host) = current_host {
                        host.hostname = Some(value.to_string());
                    }
                }
                "user" => {
                    if let Some(ref mut host) = current_host {
                        host.user = Some(value.to_string());
                    }
                }
                "port" => {
                    if let Some(ref mut host) = current_host {
                        if let Ok(port) = value.parse() {
                            host.port = Some(port);
                        }
                    }
                }
                _ => {}
            }
        }

        // Don't forget the last host
        if let Some(host) = current_host {
            if !host.name.contains('*') && !host.name.contains('?') {
                hosts.push(host);
            }
        }

        hosts
    }

    fn ssh_config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join(".ssh")
            .join("config")
    }
}

impl PluginHandler for Plugin {
    type Item = SshHost;
    const PREFIX: &'static str = "ssh ";

    fn search(&mut self, query: &str) -> Search<SshHost> {
        let query = query.to_lowercase();

        if query.is_empty() {
            // Show all hosts when no query
            return Search::Results(self.hosts.iter().take(15).cloned().collect());
        }

        let matching: Vec<SshHost> = self
            .hosts
            .iter()
            .filter(|h| h.matches(&query))
            .take(15)
            .cloned()
            .collect();

        if matching.is_empty() {
            Search::error(
                "No matching hosts",
                format!("No hosts found for '{}'", query),
            )
        } else {
            Search::Results(matching)
        }
    }

    fn row(&self, host: &SshHost) -> Row {
        Row::new(
            host.name.clone(),
            host.description(),
            IconSource::Name("utilities-terminal".to_string()),
        )
    }

    fn activate(&mut self, host: &SshHost) -> Activation {
        host.open_in_terminal();
        Activation::Close
    }

    fn context_menu(&self, _host: &SshHost) -> Vec<ContextOption> {
        vec![
            ContextOption::new(0, "Connect via SSH"),
            ContextOption::new(1, "Copy SSH command"),
            ContextOption::new(2, "Copy hostname"),
        ]
    }

    fn activate_context(&mut self, host: &SshHost, context: u32) -> Activation {
        match context {
            0 => host.open_in_terminal(),
            1 => copy_to_clipboard(&host.ssh_command()),
            2 => {
                let hostname = host.hostname.as_ref().unwrap_or(&host.name);
                copy_to_clipboard(hostname);
            }
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
    use super::*;

    #[test]
    fn test_ssh_host_description() {
        let host = SshHost {
            name: "myserver".to_string(),
            hostname: Some("192.168.1.100".to_string()),
            user: Some("admin".to_string()),
            port: Some(2222),
        };
        assert_eq!(host.description(), "admin@192.168.1.100:2222");
    }

    #[test]
    fn test_ssh_host_description_default_port() {
        let host = SshHost {
            name: "myserver".to_string(),
            hostname: Some("example.com".to_string()),
            user: Some("user".to_string()),
            port: Some(22),
        };
        assert_eq!(host.description(), "user@example.com");
    }

    #[test]
    fn test_ssh_host_ssh_command() {
        let host = SshHost {
            name: "myserver".to_string(),
            hostname: None,
            user: None,
            port: None,
        };
        assert_eq!(host.ssh_command(), "ssh myserver");
    }
}
