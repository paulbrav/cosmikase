//! SSH Hosts Pop Launcher Plugin
//!
//! A plugin for the Pop!_OS / COSMIC launcher that provides quick access to SSH
//! hosts defined in ~/.ssh/config. The launcher protocol lives in
//! `plugin-common`; this file is only the SSH-config-specific logic.

use plugin_common::{
    copy_to_clipboard, send_context, send_error_result, send_finished, send_response,
    spawn_in_terminal, ClearResponse, CloseResponse, IconSource, PluginHandler, PluginResponse,
    PluginSearchResult,
};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io;
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
}

// ============================================================================
// Plugin State
// ============================================================================

struct Plugin {
    /// Store hosts for activation by index
    results: HashMap<u32, SshHost>,
    /// Cached hosts from SSH config
    hosts: Vec<SshHost>,
}

impl Plugin {
    fn new() -> Self {
        let hosts = Self::parse_ssh_config();
        Plugin {
            results: HashMap::new(),
            hosts,
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

        // Regex patterns for SSH config parsing
        let host_re = Regex::new(r"(?i)^\s*Host\s+(.+)$").unwrap();
        let hostname_re = Regex::new(r"(?i)^\s*HostName\s+(.+)$").unwrap();
        let user_re = Regex::new(r"(?i)^\s*User\s+(.+)$").unwrap();
        let port_re = Regex::new(r"(?i)^\s*Port\s+(\d+)$").unwrap();

        for line in contents.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Check for Host directive
            if let Some(caps) = host_re.captures(line) {
                // Save previous host if any
                if let Some(host) = current_host.take() {
                    // Filter out wildcard patterns
                    if !host.name.contains('*') && !host.name.contains('?') {
                        hosts.push(host);
                    }
                }

                // Parse host names (can be multiple space-separated)
                let host_names = caps[1].split_whitespace();
                for host_name in host_names {
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
                continue;
            }

            // Parse other directives if we're in a Host block
            if let Some(ref mut host) = current_host {
                if let Some(caps) = hostname_re.captures(line) {
                    host.hostname = Some(caps[1].trim().to_string());
                } else if let Some(caps) = user_re.captures(line) {
                    host.user = Some(caps[1].trim().to_string());
                } else if let Some(caps) = port_re.captures(line) {
                    if let Ok(port) = caps[1].trim().parse() {
                        host.port = Some(port);
                    }
                }
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
    fn handle_search(&mut self, query: &str, stdout: &mut io::Stdout) {
        // Clear previous results
        self.results.clear();
        send_response(&PluginResponse::Clear(ClearResponse::Clear), stdout);

        // Strip the "ssh " prefix if present
        let search_query = query
            .strip_prefix("ssh ")
            .unwrap_or(query)
            .trim()
            .to_lowercase();

        if search_query.is_empty() {
            // Show all hosts when no query
            for (idx, host) in self.hosts.iter().take(15).enumerate() {
                let id = idx as u32;
                let search_result = PluginSearchResult {
                    id,
                    name: host.name.clone(),
                    description: host.description(),
                    keywords: None,
                    icon: Some(IconSource::Name("utilities-terminal".to_string())),
                    exec: None,
                };
                self.results.insert(id, host.clone());
                send_response(
                    &PluginResponse::Append {
                        Append: search_result,
                    },
                    stdout,
                );
            }
        } else {
            // Filter hosts by query
            let matching_hosts: Vec<&SshHost> = self
                .hosts
                .iter()
                .filter(|h| {
                    h.name.to_lowercase().contains(&search_query)
                        || h.hostname
                            .as_ref()
                            .map(|hn| hn.to_lowercase().contains(&search_query))
                            .unwrap_or(false)
                        || h.user
                            .as_ref()
                            .map(|u| u.to_lowercase().contains(&search_query))
                            .unwrap_or(false)
                })
                .collect();

            if matching_hosts.is_empty() {
                send_error_result(
                    "No matching hosts",
                    &format!("No hosts found for '{}'", search_query),
                    stdout,
                );
            } else {
                for (idx, host) in matching_hosts.into_iter().take(15).enumerate() {
                    let id = idx as u32;
                    let search_result = PluginSearchResult {
                        id,
                        name: host.name.clone(),
                        description: host.description(),
                        keywords: None,
                        icon: Some(IconSource::Name("utilities-terminal".to_string())),
                        exec: None,
                    };
                    self.results.insert(id, host.clone());
                    send_response(
                        &PluginResponse::Append {
                            Append: search_result,
                        },
                        stdout,
                    );
                }
            }
        }

        send_finished(stdout);
    }

    fn handle_activate(&mut self, id: u32, stdout: &mut io::Stdout) {
        if let Some(host) = self.results.get(&id) {
            host.open_in_terminal();
            send_response(&PluginResponse::Close(CloseResponse::Close), stdout);
        }
    }

    fn handle_context(&mut self, id: u32, stdout: &mut io::Stdout) {
        if self.results.contains_key(&id) {
            send_context(
                id,
                serde_json::json!([
                    {"id": 0, "name": "Connect via SSH"},
                    {"id": 1, "name": "Copy SSH command"},
                    {"id": 2, "name": "Copy hostname"}
                ]),
                stdout,
            );
        }
    }

    fn handle_activate_context(&mut self, id: u32, context: u32, stdout: &mut io::Stdout) {
        if let Some(host) = self.results.get(&id) {
            match context {
                0 => {
                    // Connect via SSH
                    host.open_in_terminal();
                }
                1 => {
                    // Copy SSH command
                    copy_to_clipboard(&host.ssh_command());
                }
                2 => {
                    // Copy hostname
                    let hostname = host.hostname.as_ref().unwrap_or(&host.name);
                    copy_to_clipboard(hostname);
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
