//! Man Pages Pop Launcher Plugin
//!
//! A plugin for the Pop!_OS / COSMIC launcher that provides quick access to man
//! pages using apropos for fuzzy searching. The launcher protocol lives in
//! `plugin-common`; this file is only the man-page-specific logic.

use plugin_common::{
    copy_to_clipboard, spawn_in_terminal, truncate_string, Activation, ContextOption, IconSource,
    PluginHandler, Row, Search,
};
use regex::Regex;
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

struct Plugin;

impl Plugin {
    fn new() -> Self {
        Plugin
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
    type Item = ManPage;
    const PREFIX: &'static str = "man ";

    fn search(&mut self, query: &str) -> Search<ManPage> {
        if query.is_empty() {
            // Show help when no query
            return Search::help(
                "Man Page Search",
                "Type to search man pages...",
                IconSource::Name("help-contents".to_string()),
            );
        }

        let pages = self.search_man_pages(query);

        if pages.is_empty() {
            Search::error(
                "No man pages found",
                format!("No man pages matching '{}'", query),
            )
        } else {
            Search::Results(pages.into_iter().take(15).collect())
        }
    }

    fn row(&self, page: &ManPage) -> Row {
        Row::new(
            page.display_name(),
            truncate_string(&page.description, 80),
            IconSource::Name("help-contents".to_string()),
        )
    }

    fn activate(&mut self, page: &ManPage) -> Activation {
        page.open_in_terminal();
        Activation::Close
    }

    fn context_menu(&self, _page: &ManPage) -> Vec<ContextOption> {
        vec![
            ContextOption::new(0, "Open in terminal"),
            ContextOption::new(1, "Open in browser (man.cx)"),
            ContextOption::new(2, "Copy man command"),
        ]
    }

    fn activate_context(&mut self, page: &ManPage, context: u32) -> Activation {
        match context {
            0 => page.open_in_terminal(),
            1 => {
                let _ = Command::new("xdg-open").arg(page.web_url()).spawn();
            }
            2 => copy_to_clipboard(&page.man_command()),
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
