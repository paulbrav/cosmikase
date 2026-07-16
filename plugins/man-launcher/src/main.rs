//! Man Pages Pop Launcher Plugin
//!
//! A plugin for the Pop!_OS / COSMIC launcher that provides quick access to man
//! pages using apropos for fuzzy searching. The launcher protocol lives in
//! `plugin-common`; this file is only the man-page-specific logic.

use plugin_common::{
    copy_to_clipboard, send_context, send_error_result, send_finished, send_response,
    spawn_in_terminal, truncate_string, ClearResponse, CloseResponse, IconSource, PluginHandler,
    PluginResponse, PluginSearchResult,
};
use regex::Regex;
use std::collections::HashMap;
use std::io;
use std::process::Command;

// ============================================================================
// Man Page Types
// ============================================================================

#[derive(Debug, Clone)]
struct ManPage {
    name: String,
    section: String,
    description: String,
}

impl ManPage {
    fn man_command(&self) -> String {
        format!("man {} {}", self.section, self.name)
    }

    fn display_name(&self) -> String {
        format!("{}({})", self.name, self.section)
    }

    fn web_url(&self) -> String {
        format!("https://man.cx/{}({})", self.name, self.section)
    }

    /// Open this man page inside a terminal. `section` and `name` are passed as
    /// distinct argv entries (`man <section> <name>`) — no shell, so a page name
    /// containing shell metacharacters cannot be interpreted as a command.
    fn open_in_terminal(&self) {
        spawn_in_terminal(&["man", &self.section, &self.name]);
    }
}

// ============================================================================
// Plugin State
// ============================================================================

struct Plugin {
    /// Store man pages for activation by index
    results: HashMap<u32, ManPage>,
}

impl Plugin {
    fn new() -> Self {
        Plugin {
            results: HashMap::new(),
        }
    }

    fn search_man_pages(&self, query: &str) -> Vec<ManPage> {
        // Use apropos to search man pages
        // -s limits to useful sections: 1=commands, 2=syscalls, 3=library, 5=files, 7=misc, 8=admin
        let output = Command::new("apropos")
            .args(["-s", "1,2,3,5,7,8", query])
            .output();

        let output = match output {
            Ok(o) => o,
            Err(_) => return Vec::new(),
        };

        if !output.status.success() {
            return Vec::new();
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_apropos_output(&stdout)
    }

    fn parse_apropos_output(&self, output: &str) -> Vec<ManPage> {
        let mut pages = Vec::new();

        // apropos output format: "name (section) - description"
        // or "name, alias (section) - description"
        let re = Regex::new(r"^([^\s,]+)(?:,\s*[^\(]+)?\s+\((\d+[a-z]*)\)\s+-\s+(.+)$").unwrap();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some(caps) = re.captures(line) {
                let name = caps[1].to_string();
                let section = caps[2].to_string();
                let description = caps[3].to_string();

                pages.push(ManPage {
                    name,
                    section,
                    description,
                });
            }
        }

        pages
    }
}

impl PluginHandler for Plugin {
    fn handle_search(&mut self, query: &str, stdout: &mut io::Stdout) {
        // Clear previous results
        self.results.clear();
        send_response(&PluginResponse::Clear(ClearResponse::Clear), stdout);

        // Strip the "man " prefix if present
        let search_query = query.strip_prefix("man ").unwrap_or(query).trim();

        if search_query.is_empty() {
            // Show help when no query
            let result = PluginSearchResult {
                id: 0,
                name: "Man Page Search".to_string(),
                description: "Type to search man pages...".to_string(),
                keywords: None,
                icon: Some(IconSource::Name("help-contents".to_string())),
                exec: None,
            };
            send_response(&PluginResponse::Append { Append: result }, stdout);
            send_finished(stdout);
            return;
        }

        // Search for man pages
        let pages = self.search_man_pages(search_query);

        if pages.is_empty() {
            send_error_result(
                "No man pages found",
                &format!("No man pages matching '{}'", search_query),
                stdout,
            );
        } else {
            for (idx, page) in pages.into_iter().take(15).enumerate() {
                let id = idx as u32;
                let search_result = PluginSearchResult {
                    id,
                    name: page.display_name(),
                    description: truncate_string(&page.description, 80),
                    keywords: None,
                    icon: Some(IconSource::Name("help-contents".to_string())),
                    exec: None,
                };
                self.results.insert(id, page);
                send_response(
                    &PluginResponse::Append {
                        Append: search_result,
                    },
                    stdout,
                );
            }
        }

        send_finished(stdout);
    }

    fn handle_activate(&mut self, id: u32, stdout: &mut io::Stdout) {
        if let Some(page) = self.results.get(&id) {
            page.open_in_terminal();
            send_response(&PluginResponse::Close(CloseResponse::Close), stdout);
        }
    }

    fn handle_context(&mut self, id: u32, stdout: &mut io::Stdout) {
        if self.results.contains_key(&id) {
            send_context(
                id,
                serde_json::json!([
                    {"id": 0, "name": "Open in terminal"},
                    {"id": 1, "name": "Open in browser (man.cx)"},
                    {"id": 2, "name": "Copy man command"}
                ]),
                stdout,
            );
        }
    }

    fn handle_activate_context(&mut self, id: u32, context: u32, stdout: &mut io::Stdout) {
        if let Some(page) = self.results.get(&id) {
            match context {
                0 => {
                    // Open in terminal
                    page.open_in_terminal();
                }
                1 => {
                    // Open in browser
                    let _ = Command::new("xdg-open").arg(page.web_url()).spawn();
                }
                2 => {
                    // Copy man command
                    copy_to_clipboard(&page.man_command());
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
    fn test_man_page_display_name() {
        let page = ManPage {
            name: "ls".to_string(),
            section: "1".to_string(),
            description: "list directory contents".to_string(),
        };
        assert_eq!(page.display_name(), "ls(1)");
    }

    #[test]
    fn test_man_page_man_command() {
        let page = ManPage {
            name: "printf".to_string(),
            section: "3".to_string(),
            description: "formatted output conversion".to_string(),
        };
        assert_eq!(page.man_command(), "man 3 printf");
    }

    #[test]
    fn test_man_page_web_url() {
        let page = ManPage {
            name: "grep".to_string(),
            section: "1".to_string(),
            description: "print lines matching a pattern".to_string(),
        };
        assert_eq!(page.web_url(), "https://man.cx/grep(1)");
    }

    #[test]
    fn test_parse_apropos_output() {
        let plugin = Plugin::new();
        let output = "ls (1)                - list directory contents\nfind (1)              - search for files in a directory hierarchy";
        let pages = plugin.parse_apropos_output(output);
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].name, "ls");
        assert_eq!(pages[0].section, "1");
        assert_eq!(pages[1].name, "find");
    }
}
