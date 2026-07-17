//! Clipboard History Pop Launcher Plugin
//!
//! A plugin for the Pop!_OS / COSMIC launcher that provides access to clipboard
//! history via cliphist integration. The launcher protocol lives in
//! `plugin-common`; this file is only the cliphist-specific logic.

use plugin_common::{
    command_exists, truncate_string, Activation, ContextOption, IconSource, PluginHandler, Row,
    Search,
};
use std::io::Write;
use std::process::{Command, Stdio};

// ============================================================================
// Clipboard Entry Types
// ============================================================================

#[derive(Debug, Clone)]
struct ClipboardEntry {
    /// The raw line from cliphist (includes ID prefix)
    raw_line: String,
    /// Display content (truncated for UI)
    display: String,
    /// Whether this is an image entry
    is_image: bool,
}

// ============================================================================
// Plugin State
// ============================================================================

struct Plugin;

impl Plugin {
    fn new() -> Self {
        Plugin
    }

    fn get_clipboard_history(&self) -> Vec<ClipboardEntry> {
        // Run cliphist list to get clipboard history
        let output = Command::new("cliphist").arg("list").output();

        let output = match output {
            Ok(o) => o,
            Err(_) => return Vec::new(),
        };

        if !output.status.success() {
            return Vec::new();
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_cliphist_output(&stdout)
    }

    fn parse_cliphist_output(&self, output: &str) -> Vec<ClipboardEntry> {
        let mut entries = Vec::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // cliphist format: "ID\tCONTENT" or "[binary data]" for images
            let is_image = line.contains("[[ binary data ]]");

            let display = if is_image {
                "[Image]".to_string()
            } else {
                // Extract content after the first tab (ID is before tab)
                let content = line.split('\t').nth(1).unwrap_or(line);
                // Clean up the display: collapse whitespace, truncate
                let cleaned: String = content
                    .chars()
                    .map(|c| if c.is_whitespace() { ' ' } else { c })
                    .collect();
                truncate_string(&cleaned, 80)
            };

            entries.push(ClipboardEntry {
                raw_line: line.to_string(),
                display,
                is_image,
            });
        }

        entries
    }

    /// Feed an entry's raw cliphist line to `cliphist decode` (over stdin), then
    /// pipe the decoded bytes to `wl-copy` (over stdin). No shell is involved,
    /// so the untrusted clipboard content can never be interpreted as shell
    /// syntax — this replaces the previous `sh -c "echo '{raw_line}' | ..."`,
    /// which was vulnerable to injection via a single quote in the content.
    fn paste_entry(&self, entry: &ClipboardEntry) {
        let mut decode = match Command::new("cliphist")
            .arg("decode")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => return,
        };

        if let Some(mut stdin) = decode.stdin.take() {
            let _ = stdin.write_all(entry.raw_line.as_bytes());
            // stdin dropped here, closing the pipe so cliphist can finish.
        }

        let decoded = match decode.wait_with_output() {
            Ok(o) if o.status.success() => o.stdout,
            _ => return,
        };

        if let Ok(mut copy) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
            if let Some(mut stdin) = copy.stdin.take() {
                let _ = stdin.write_all(&decoded);
            }
            let _ = copy.wait();
        }
    }

    /// Delete an entry by feeding its raw line to `cliphist delete` over stdin.
    /// As with `paste_entry`, no shell interpolation of untrusted data occurs.
    fn delete_entry(&self, entry: &ClipboardEntry) {
        if let Ok(mut child) = Command::new("cliphist")
            .arg("delete")
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(entry.raw_line.as_bytes());
            }
            let _ = child.wait();
        }
    }
}

impl PluginHandler for Plugin {
    type Item = ClipboardEntry;
    const PREFIX: &'static str = "clip ";

    fn search(&mut self, query: &str) -> Search<ClipboardEntry> {
        let query = query.to_lowercase();

        // Check if cliphist is available
        if !command_exists("cliphist") {
            return Search::error(
                "cliphist not found",
                "Install cliphist: go install go.senan.xyz/cliphist@latest",
            );
        }

        // Get clipboard history
        let entries = self.get_clipboard_history();

        if entries.is_empty() {
            return Search::error("Clipboard empty", "No clipboard history available");
        }

        // Filter entries by query if provided
        let filtered: Vec<ClipboardEntry> = if query.is_empty() {
            entries
        } else {
            entries
                .into_iter()
                .filter(|e| e.display.to_lowercase().contains(&query))
                .collect()
        };

        if filtered.is_empty() {
            Search::error(
                "No matches",
                format!("No clipboard entries matching '{}'", query),
            )
        } else {
            Search::Results(filtered.into_iter().take(20).collect())
        }
    }

    fn row(&self, entry: &ClipboardEntry) -> Row {
        let (icon, description) = if entry.is_image {
            ("image-x-generic", "Image from clipboard")
        } else {
            ("edit-paste", "Text entry")
        };
        Row::new(
            entry.display.clone(),
            description,
            IconSource::Name(icon.to_string()),
        )
    }

    fn activate(&mut self, entry: &ClipboardEntry) -> Activation {
        self.paste_entry(entry);
        Activation::Close
    }

    fn context_menu(&self, _entry: &ClipboardEntry) -> Vec<ContextOption> {
        vec![
            ContextOption::new(0, "Paste to clipboard"),
            ContextOption::new(1, "Delete from history"),
        ]
    }

    fn activate_context(&mut self, entry: &ClipboardEntry, context: u32) -> Activation {
        match context {
            0 => self.paste_entry(entry),
            1 => self.delete_entry(entry),
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
    fn test_parse_cliphist_output() {
        let plugin = Plugin::new();
        let output = "1\tHello World\n2\tAnother entry\n3\t[[ binary data ]]";
        let entries = plugin.parse_cliphist_output(output);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].display, "Hello World");
        assert!(!entries[0].is_image);
        assert_eq!(entries[2].display, "[Image]");
        assert!(entries[2].is_image);
    }
}
