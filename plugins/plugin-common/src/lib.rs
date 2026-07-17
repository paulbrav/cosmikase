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
use std::collections::HashMap;
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
// Plugin lifecycle
// ============================================================================

/// The display row emitted for one activatable result. `id`, `keywords`, and
/// `exec` are supplied uniformly by the lifecycle; a plugin fills in only the
/// three fields that vary per result.
#[derive(Debug)]
pub struct Row {
    pub name: String,
    pub description: String,
    pub icon: IconSource,
}

impl Row {
    pub fn new(name: impl Into<String>, description: impl Into<String>, icon: IconSource) -> Self {
        Row {
            name: name.into(),
            description: description.into(),
            icon,
        }
    }
}

/// One context-menu entry (`id` + label) offered for a result.
#[derive(Debug)]
pub struct ContextOption {
    pub id: u32,
    pub name: String,
}

impl ContextOption {
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        ContextOption {
            id,
            name: name.into(),
        }
    }
}

/// Whether an activation should close the launcher. Returned by
/// [`PluginHandler::activate`] / [`PluginHandler::activate_context`] so a plugin
/// can deliberately stay open (e.g. a locked vault that produced no result).
#[derive(Debug, Clone, Copy)]
pub enum Activation {
    /// Close the launcher after handling this activation.
    Close,
    /// Leave the launcher open.
    KeepOpen,
}

/// The outcome of a [`PluginHandler::search`] call. The lifecycle turns each
/// variant into the correct Clear/Append/Finished sequence: only `Results` rows
/// are stored and therefore activatable, so `Help`/`Error` rows stay
/// non-activatable by construction (they are never inserted into the store).
pub enum Search<T> {
    /// Activatable results, rendered via [`PluginHandler::row`]. An empty vec
    /// yields a `Finished` with no rows.
    Results(Vec<T>),
    /// A single non-activatable informational row.
    Help(PluginSearchResult),
    /// A single non-activatable error row (title, message).
    Error(String, String),
}

impl<T> Search<T> {
    /// Build a `Help` informational row (fixed `id` 0, no keywords/exec).
    pub fn help(name: impl Into<String>, description: impl Into<String>, icon: IconSource) -> Self {
        Search::Help(PluginSearchResult {
            id: 0,
            name: name.into(),
            description: description.into(),
            keywords: None,
            icon: Some(icon),
            exec: None,
        })
    }

    /// Build an `Error` row (title + message).
    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        Search::Error(title.into(), message.into())
    }
}

/// Domain behaviour each plugin provides. The blocking stdin/stdout dispatch
/// loop, the id-keyed result store, the prefix strip, and the whole
/// Clear/Append/Finished/Close protocol dance are owned by the lifecycle
/// ([`PluginHandler::run`]); an implementor writes only domain logic.
pub trait PluginHandler {
    /// The domain object stored per activatable result and handed back to
    /// [`row`](Self::row), [`activate`](Self::activate), and the context handlers.
    type Item;

    /// The launcher prefix stripped (then trimmed) from every query before
    /// [`search`](Self::search) sees it (e.g. `"ssh "`).
    const PREFIX: &'static str;

    /// Produce the search outcome for `query` (already prefix-stripped and
    /// trimmed).
    fn search(&mut self, query: &str) -> Search<Self::Item>;

    /// Render the display row for one stored `Item`.
    fn row(&self, item: &Self::Item) -> Row;

    /// Handle default activation (Enter) of a stored `Item`.
    fn activate(&mut self, item: &Self::Item) -> Activation;

    /// Context-menu options for a stored `Item` (empty = no menu). Defaults to
    /// none.
    fn context_menu(&self, _item: &Self::Item) -> Vec<ContextOption> {
        Vec::new()
    }

    /// Handle activation of context option `context` on a stored `Item`.
    /// Defaults to leaving the launcher open.
    fn activate_context(&mut self, _item: &Self::Item, _context: u32) -> Activation {
        Activation::KeepOpen
    }

    /// Run the blocking dispatch loop until the launcher sends `Exit` or stdin
    /// reaches EOF. Malformed lines are skipped rather than fatal.
    fn run(self)
    where
        Self: Sized,
    {
        let mut runner = Runner {
            plugin: self,
            results: HashMap::new(),
        };
        runner.dispatch();
    }
}

/// Owns the id-keyed result store and drives the wire protocol so that plugin
/// code never touches it.
struct Runner<P: PluginHandler> {
    plugin: P,
    results: HashMap<u32, P::Item>,
}

impl<P: PluginHandler> Runner<P> {
    fn dispatch(&mut self) {
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
                Request::Search { Search: query } => self.on_search(&query, &mut stdout),
                Request::Activate { Activate: id } => self.on_activate(id, &mut stdout),
                Request::Simple(SimpleRequest::Exit) => break,
                Request::Simple(SimpleRequest::Interrupt) => send_finished(&mut stdout),
                Request::Context { Context: id } => self.on_context(id, &mut stdout),
                Request::ActivateContext {
                    ActivateContext: data,
                } => self.on_activate_context(data.id, data.context, &mut stdout),
                _ => send_finished(&mut stdout),
            }
        }
    }

    fn on_search(&mut self, query: &str, stdout: &mut io::Stdout) {
        self.results.clear();
        send_response(&PluginResponse::Clear(ClearResponse::Clear), stdout);

        let stripped = query.strip_prefix(P::PREFIX).unwrap_or(query).trim();

        match self.plugin.search(stripped) {
            Search::Results(items) => {
                for (idx, item) in items.into_iter().enumerate() {
                    let id = idx as u32;
                    let Row {
                        name,
                        description,
                        icon,
                    } = self.plugin.row(&item);
                    let result = PluginSearchResult {
                        id,
                        name,
                        description,
                        keywords: None,
                        icon: Some(icon),
                        exec: None,
                    };
                    self.results.insert(id, item);
                    send_response(&PluginResponse::Append { Append: result }, stdout);
                }
            }
            Search::Help(row) => {
                send_response(&PluginResponse::Append { Append: row }, stdout);
            }
            Search::Error(title, message) => {
                send_error_result(&title, &message, stdout);
            }
        }

        send_finished(stdout);
    }

    fn on_activate(&mut self, id: u32, stdout: &mut io::Stdout) {
        // Destructure so the store (`results`) and the domain handler (`plugin`)
        // are borrowed as disjoint fields — `activate` needs `&mut plugin` while
        // the item borrows `results`.
        let Runner { plugin, results } = self;
        if let Some(item) = results.get(&id) {
            if let Activation::Close = plugin.activate(item) {
                send_response(&PluginResponse::Close(CloseResponse::Close), stdout);
            }
        }
    }

    fn on_context(&mut self, id: u32, stdout: &mut io::Stdout) {
        if let Some(item) = self.results.get(&id) {
            let options = self.plugin.context_menu(item);
            if !options.is_empty() {
                let options = serde_json::Value::Array(
                    options
                        .iter()
                        .map(|o| serde_json::json!({ "id": o.id, "name": o.name }))
                        .collect(),
                );
                send_context(id, options, stdout);
            }
        }
    }

    fn on_activate_context(&mut self, id: u32, context: u32, stdout: &mut io::Stdout) {
        let Runner { plugin, results } = self;
        if let Some(item) = results.get(&id) {
            if let Activation::Close = plugin.activate_context(item, context) {
                send_response(&PluginResponse::Close(CloseResponse::Close), stdout);
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
