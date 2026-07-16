//! Clipboard History Pop Launcher Plugin
//!
//! A plugin for the Pop!_OS / COSMIC launcher that provides access to clipboard
//! history via cliphist integration. The launcher protocol lives in
//! `plugin-common`; this file is only the cliphist-specific logic.

use plugin_common::{
    command_exists, send_context, send_error_result, send_finished, send_response, truncate_string,
    ClearResponse, CloseResponse, IconSource, PluginHandler, PluginResponse, PluginSearchResult,
};
use std::collections::HashMap;
use std::io::{self, Write};
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

struct Plugin {
    /// Store clipboard entries for activation by index
    results: HashMap<u32, ClipboardEntry>,
}

impl Plugin {
    fn new() -> Self {
        Plugin {
            results: HashMap::new(),
        }
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
    fn handle_search(&mut self, query: &str, stdout: &mut io::Stdout) {
        // Clear previous results
        self.results.clear();
        send_response(&PluginResponse::Clear(ClearResponse::Clear), stdout);

        // Strip the "clip " prefix if present
        let search_query = query
            .strip_prefix("clip ")
            .unwrap_or(query)
            .trim()
            .to_lowercase();

        // Check if cliphist is available
        if !command_exists("cliphist") {
            send_error_result(
                "cliphist not found",
                "Install cliphist: go install go.senan.xyz/cliphist@latest",
                stdout,
            );
            send_finished(stdout);
            return;
        }

        // Get clipboard history
        let entries = self.get_clipboard_history();

        if entries.is_empty() {
            send_error_result("Clipboard empty", "No clipboard history available", stdout);
            send_finished(stdout);
            return;
        }

        // Filter entries by query if provided
        let filtered_entries: Vec<&ClipboardEntry> = if search_query.is_empty() {
            entries.iter().collect()
        } else {
            entries
                .iter()
                .filter(|e| e.display.to_lowercase().contains(&search_query))
                .collect()
        };

        if filtered_entries.is_empty() {
            send_error_result(
                "No matches",
                &format!("No clipboard entries matching '{}'", search_query),
                stdout,
            );
        } else {
            for (idx, entry) in filtered_entries.into_iter().take(20).enumerate() {
                let id = idx as u32;
                let icon = if entry.is_image {
                    "image-x-generic"
                } else {
                    "edit-paste"
                };

                let search_result = PluginSearchResult {
                    id,
                    name: entry.display.clone(),
                    description: if entry.is_image {
                        "Image from clipboard".to_string()
                    } else {
                        "Text entry".to_string()
                    },
                    keywords: None,
                    icon: Some(IconSource::Name(icon.to_string())),
                    exec: None,
                };
                self.results.insert(id, entry.clone());
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
        if let Some(entry) = self.results.get(&id) {
            self.paste_entry(entry);
            send_response(&PluginResponse::Close(CloseResponse::Close), stdout);
        }
    }

    fn handle_context(&mut self, id: u32, stdout: &mut io::Stdout) {
        if self.results.contains_key(&id) {
            send_context(
                id,
                serde_json::json!([
                    {"id": 0, "name": "Paste to clipboard"},
                    {"id": 1, "name": "Delete from history"}
                ]),
                stdout,
            );
        }
    }

    fn handle_activate_context(&mut self, id: u32, context: u32, stdout: &mut io::Stdout) {
        if let Some(entry) = self.results.get(&id) {
            match context {
                0 => {
                    // Paste to clipboard
                    self.paste_entry(entry);
                }
                1 => {
                    // Delete from history
                    self.delete_entry(entry);
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
