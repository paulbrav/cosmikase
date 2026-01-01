"""Textual TUI for browsing and applying omarchy-pop themes."""

from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path
from typing import Iterable, List, Optional

from textual import on
from textual.app import App, ComposeResult
from textual.binding import Binding
from textual.reactive import reactive
from textual.widgets import Footer, Header, OptionList, Static
from textual.widgets.option_list import Option


def _unique_dirs(candidates: Iterable[Path]) -> List[Path]:
    seen = set()
    unique: List[Path] = []
    for path in candidates:
        resolved = path.expanduser()
        key = resolved.resolve() if resolved.exists() else resolved
        if key in seen or not resolved.is_dir():
            continue
        seen.add(key)
        unique.append(resolved)
    return unique


def discover_theme_dirs() -> List[Path]:
    env_dir = os.environ.get("THEMES_DIR")
    repo_root = _find_repo_root()
    candidates = [
        Path(env_dir) if env_dir else None,
        repo_root / "themes" if repo_root else None,
        Path.cwd() / "themes",
        Path.home() / ".local" / "share" / "omarchy-pop" / "themes",
    ]
    return _unique_dirs(path for path in candidates if path is not None)


def list_theme_names(base: Optional[Path]) -> List[str]:
    if base is None or not base.is_dir():
        return []
    return sorted(entry.name for entry in base.iterdir() if entry.is_dir())


def _find_repo_root() -> Optional[Path]:
    """Walk up from this file to find the repo root (contains themes/ and bin/)."""
    current = Path(__file__).resolve().parent
    for _ in range(5):  # Limit search depth
        if (current / "themes").is_dir() and (current / "bin").is_dir():
            return current
        if current.parent == current:
            break
        current = current.parent
    # Also check cwd
    cwd = Path.cwd()
    if (cwd / "themes").is_dir() and (cwd / "bin").is_dir():
        return cwd
    return None


def find_theme_cli() -> Optional[str]:
    repo_root = _find_repo_root()
    candidates = [
        os.environ.get("THEME_CLI"),
        str(repo_root / "bin" / "omarchy-pop-theme") if repo_root else None,
        shutil.which("omarchy-pop-theme"),
        str(Path.home() / ".local" / "bin" / "omarchy-pop-theme"),
    ]
    for candidate in candidates:
        if not candidate:
            continue
        if shutil.which(candidate):
            return candidate
        path = Path(candidate).expanduser()
        if path.is_file() and os.access(path, os.X_OK):
            return str(path)
    return None


class ThemeTui(App):
    """Minimal Textual app to browse and apply omarchy-pop themes."""

    CSS = """
    Screen {
        align: center middle;
    }

    #container {
        width: 80%;
        max-width: 90;
        height: 70%;
        max-height: 40;
        border: round $secondary;
        padding: 1 2;
        layout: vertical;
    }

    #path {
        color: $text-muted;
    }

    OptionList {
        height: 100%;
        border: tall $surface;
    }

    #status {
        height: 3;
        border: heavy $surface;
        padding: 0 1;
    }
    """

    BINDINGS = [
        Binding("r", "refresh", "Refresh list"),
        Binding("q", "quit", "Quit"),
    ]

    status = reactive("Select a theme and press Enter to apply.")

    def __init__(self) -> None:
        super().__init__()
        self.theme_dirs: List[Path] = discover_theme_dirs()
        self.active_dir: Optional[Path] = self.theme_dirs[0] if self.theme_dirs else None

    def compose(self) -> ComposeResult:
        yield Header(show_clock=True)
        with Static(id="container"):
            path_text = (
                f"Themes directory: {self.active_dir}"
                if self.active_dir
                else "No theme directory found. Set THEMES_DIR or run 'make install'."
            )
            yield Static(path_text, id="path")
            self.option_list = OptionList()
            yield self.option_list
            yield Static("Select a theme and press Enter to apply.", id="status")
        yield Footer()

    def on_mount(self) -> None:
        self._reload_options()
        self.option_list.focus()

    def watch_status(self, status: str) -> None:
        try:
            self.query_one("#status", Static).update(status)
        except Exception:
            pass  # Widget not yet mounted

    def _reload_options(self) -> None:
        self.option_list.clear_options()
        names = list_theme_names(self.active_dir)
        for name in names:
            self.option_list.add_option(Option(name, id=name))
        if not names:
            self.status = "No themes available. Run 'make install' to populate themes."

    def action_refresh(self) -> None:
        self.theme_dirs = discover_theme_dirs()
        self.active_dir = self.theme_dirs[0] if self.theme_dirs else None
        path_text = (
            f"Themes directory: {self.active_dir}"
            if self.active_dir
            else "No theme directory found. Set THEMES_DIR or run 'make install'."
        )
        self.query_one("#path", Static).update(path_text)
        self.status = "Theme list refreshed."
        self._reload_options()

    def _apply_theme(self, theme: str) -> None:
        cli = find_theme_cli()
        if not cli:
            self.status = "omarchy-pop-theme not found. Run 'make install' first."
            return
        self.status = f"Applying '{theme}'..."
        result = subprocess.run([cli, theme], capture_output=True, text=True)
        if result.returncode == 0:
            self.status = f"Applied '{theme}'."
        else:
            detail = result.stderr.strip() or result.stdout.strip() or str(result.returncode)
            self.status = f"Failed to apply '{theme}': {detail}"

    @on(OptionList.OptionSelected)
    def handle_option_selected(self, event: OptionList.OptionSelected) -> None:
        event.stop()
        self._apply_theme(event.option.prompt)


def run() -> None:
    ThemeTui().run()


if __name__ == "__main__":
    run()
