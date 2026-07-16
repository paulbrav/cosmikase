//! Shared protocol types and helpers for Pop!_OS / COSMIC launcher plugins.
//!
//! Every launcher plugin speaks the same newline-delimited JSON protocol with
//! the launcher (pop-launcher / cosmic-launcher): requests arrive on stdin,
//! responses go out on stdout. This crate owns that protocol block — which was
//! previously copy-pasted byte-for-byte into every plugin's `main.rs` — plus
//! the blocking dispatch loop and the small process helpers (clipboard,
//! terminal spawning) the plugins share. A plugin now implements
//! [`PluginHandler`] and calls [`PluginHandler::run`].

use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};

// ============================================================================
// Protocol: requests received from the launcher on stdin
// ============================================================================

/// A single request line sent by the launcher.
///
/// `#[serde(untagged)]` matches whichever externally-tagged shape the launcher
/// emits (`{"Search": "..."}`, `{"Activate": 3}`, the bare `"Exit"` string,
/// etc.). Field names intentionally match the wire format, hence the
/// `non_snake_case` allowance.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(non_snake_case)]
pub enum Request {
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
pub struct ActivateContextData {
    pub id: u32,
    pub context: u32,
}

#[derive(Debug, Deserialize)]
pub enum SimpleRequest {
    Exit,
    Interrupt,
}

// ============================================================================
// Protocol: responses written to the launcher on stdout
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(untagged)]
#[allow(non_snake_case)]
pub enum PluginResponse {
    Append { Append: PluginSearchResult },
    Clear(ClearResponse),
    Close(CloseResponse),
    Fill { Fill: String },
    Finished(FinishedResponse),
}

#[derive(Debug, Serialize)]
pub enum ClearResponse {
    Clear,
}

#[derive(Debug, Serialize)]
pub enum CloseResponse {
    Close,
}

#[derive(Debug, Serialize)]
pub enum FinishedResponse {
    Finished,
}

#[derive(Debug, Serialize)]
pub struct PluginSearchResult {
    pub id: u32,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<IconSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exec: Option<String>,
}

#[derive(Debug, Serialize)]
pub enum IconSource {
    Name(String),
    Mime(String),
}

// ============================================================================
// Dispatch loop
// ============================================================================

/// Behaviour each plugin provides. The blocking stdin/stdout dispatch loop is
/// supplied by [`PluginHandler::run`]; a plugin only implements the handlers it
/// needs (`handle_context` / `handle_activate_context` have no-op defaults).
pub trait PluginHandler {
    /// Handle a `Search` request. `query` still carries the launcher prefix
    /// (e.g. `"ssh foo"`); strip it in the implementation.
    fn handle_search(&mut self, query: &str, stdout: &mut io::Stdout);

    /// Handle default activation (Enter) of result `id`.
    fn handle_activate(&mut self, id: u32, stdout: &mut io::Stdout);

    /// Handle a request for the context menu of result `id`.
    fn handle_context(&mut self, _id: u32, stdout: &mut io::Stdout) {
        send_finished(stdout);
    }

    /// Handle activation of context-menu option `context` on result `id`.
    fn handle_activate_context(&mut self, _id: u32, _context: u32, _stdout: &mut io::Stdout) {}

    /// Run the blocking dispatch loop until the launcher sends `Exit` or stdin
    /// reaches EOF. Malformed lines are skipped rather than fatal.
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
                Request::Search { Search: query } => self.handle_search(&query, &mut stdout),
                Request::Activate { Activate: id } => self.handle_activate(id, &mut stdout),
                Request::Simple(SimpleRequest::Exit) => break,
                Request::Simple(SimpleRequest::Interrupt) => send_finished(&mut stdout),
                Request::Context { Context: id } => self.handle_context(id, &mut stdout),
                Request::ActivateContext {
                    ActivateContext: data,
                } => self.handle_activate_context(data.id, data.context, &mut stdout),
                _ => send_finished(&mut stdout),
            }
        }
    }
}

// ============================================================================
// Response helpers
// ============================================================================

/// Serialize and write a single protocol response, flushing immediately.
pub fn send_response(response: &PluginResponse, stdout: &mut io::Stdout) {
    if let Ok(json) = serde_json::to_string(response) {
        let _ = writeln!(stdout, "{}", json);
        let _ = stdout.flush();
    }
}

/// Signal the launcher that this batch of results is complete.
pub fn send_finished(stdout: &mut io::Stdout) {
    send_response(
        &PluginResponse::Finished(FinishedResponse::Finished),
        stdout,
    );
}

/// Append a single error row (`title` + `message`) to the result list.
pub fn send_error_result(title: &str, message: &str, stdout: &mut io::Stdout) {
    let result = PluginSearchResult {
        id: 999,
        name: title.to_string(),
        description: message.to_string(),
        keywords: None,
        icon: Some(IconSource::Name("dialog-error".to_string())),
        exec: None,
    };
    send_response(&PluginResponse::Append { Append: result }, stdout);
}

/// Emit a context-menu descriptor for result `id`. `options` is a JSON array of
/// `{ "id": u32, "name": String }` objects. The Context response is not part of
/// [`PluginResponse`]; the launcher expects this raw `{"Context": {...}}` shape.
pub fn send_context(id: u32, options: serde_json::Value, stdout: &mut io::Stdout) {
    let context_response = serde_json::json!({
        "Context": { "id": id, "options": options }
    });
    let _ = writeln!(stdout, "{}", context_response);
    let _ = stdout.flush();
}

// ============================================================================
// Process helpers
// ============================================================================

/// Return true if `program` resolves on `PATH`.
pub fn command_exists(program: &str) -> bool {
    Command::new("which")
        .arg(program)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Copy `text` to the system clipboard, preferring Wayland (`wl-copy`) and
/// falling back to X11 (`xclip`). In both cases `text` is written over stdin,
/// never through a shell and never as an argv entry, so a payload beginning with
/// `-` (e.g. a valid password) can never be parsed as an option or shell syntax.
pub fn copy_to_clipboard(text: &str) {
    // wl-copy (Wayland). Only treat it as done on a genuine success exit — the
    // binary can exist yet fail at runtime (e.g. under X11 with no Wayland
    // display), in which case we must fall through to xclip.
    if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        if child.wait().map(|s| s.success()).unwrap_or(false) {
            return;
        }
    }

    if let Ok(mut child) = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}

/// Launch `command` (program followed by its arguments) inside the first
/// available terminal emulator, returning true once one is spawned.
///
/// Each element of `command` is passed as a distinct argv entry, so untrusted
/// content (a man-page name, an SSH host alias) can never be reinterpreted as
/// shell syntax — there is no shell in the chain.
///
/// Terminals are tried in daily-driver order: cosmic-term first (the COSMIC
/// default), then ghostty (native binary, then the snap), then generic
/// fallbacks. kitty and alacritty were intentionally dropped (project decision:
/// cosmic-term + ghostty only).
pub fn spawn_in_terminal(command: &[&str]) -> bool {
    // (launcher program, fixed leading args); `command` is appended verbatim.
    let candidates: &[(&str, &[&str])] = &[
        ("cosmic-term", &["-e"]),
        ("ghostty", &["-e"]),
        ("snap", &["run", "ghostty", "--", "-e"]),
        ("gnome-terminal", &["--"]),
        ("xterm", &["-e"]),
    ];

    for (program, prefix) in candidates {
        if !command_exists(program) {
            continue;
        }
        let mut argv: Vec<&str> = prefix.to_vec();
        argv.extend_from_slice(command);
        if Command::new(program).args(&argv).spawn().is_ok() {
            return true;
        }
    }
    false
}

// ============================================================================
// Misc helpers
// ============================================================================

/// Truncate `s` to at most `max_chars` characters (char-safe, never splits a
/// multi-byte codepoint), appending an ellipsis when shortened.
pub fn truncate_string(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else {
        chars[..max_chars.saturating_sub(3)]
            .iter()
            .collect::<String>()
            + "..."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short() {
        assert_eq!(truncate_string("hello", 10), "hello");
    }

    #[test]
    fn truncate_exact() {
        assert_eq!(truncate_string("hello", 5), "hello");
    }

    #[test]
    fn truncate_long() {
        assert_eq!(truncate_string("hello world", 8), "hello...");
    }

    #[test]
    fn truncate_unicode() {
        // Must not panic on multi-byte characters.
        assert_eq!(truncate_string("héllo wörld", 8), "héllo...");
    }

    #[test]
    fn search_request_parses() {
        let req: Request = serde_json::from_str(r#"{"Search":"foo"}"#).unwrap();
        assert!(matches!(req, Request::Search { Search } if Search == "foo"));
    }

    #[test]
    fn exit_request_parses() {
        let req: Request = serde_json::from_str(r#""Exit""#).unwrap();
        assert!(matches!(req, Request::Simple(SimpleRequest::Exit)));
    }

    #[test]
    fn append_response_serializes() {
        let response = PluginResponse::Append {
            Append: PluginSearchResult {
                id: 1,
                name: "name".to_string(),
                description: "desc".to_string(),
                keywords: None,
                icon: None,
                exec: None,
            },
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"Append\""));
        assert!(json.contains("\"name\":\"name\""));
    }
}
