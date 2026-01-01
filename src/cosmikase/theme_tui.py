"""Textual TUI for browsing and applying cosmikase themes."""

from __future__ import annotations

import contextlib
import subprocess
from pathlib import Path

from rich.style import Style
from rich.text import Text
from textual import on
from textual.app import App, ComposeResult
from textual.binding import Binding
from textual.containers import Horizontal
from textual.reactive import reactive
from textual.widgets import Footer, Header, OptionList, Static
from textual.widgets.option_list import Option
from textual.worker import Worker, WorkerState, work

from cosmikase.themes import discover_theme_dirs, find_theme_cli, list_themes, load_manifest


class ThemePreview(Static):
    """Shows color swatches for selected theme."""
    
    def update_preview(self, theme_path: Path) -> None:
        try:
            manifest = load_manifest(theme_path)
            text = Text()
            text.append(f"Theme: {manifest.name}\n", style="bold")
            text.append(f"Variant: {manifest.variant}\n\n")
            
            if manifest.colors:
                text.append("Colors:\n")
                for name, hex_color in manifest.colors.items():
                    # Ensure hex_color is valid for Rich
                    clean_color = hex_color if hex_color.startswith("#") else f"#{hex_color}"
                    text.append("██ ", style=Style(color=clean_color))
                    text.append(f"{name}: {hex_color}\n")
            else:
                text.append("No colors defined in manifest.\n")
                
            if manifest.cursor_theme:
                text.append(f"\nCursor: {manifest.cursor_theme}\n")
            if manifest.wallpaper:
                text.append(f"Wallpaper: {manifest.wallpaper}\n")
                
            self.update(text)
        except Exception as e:
            self.update(f"Error loading preview: {e}")


class ThemeTui(App):
    """Minimal Textual app to browse and apply cosmikase themes."""

    CSS = """
    Screen {
        align: center middle;
    }

    #container {
        width: 90%;
        max-width: 120;
        height: 80%;
        max-height: 50;
        border: round $secondary;
        padding: 1 2;
        layout: vertical;
    }

    #path {
        color: $text-muted;
        margin-bottom: 1;
    }

    #main-content {
        height: 100%;
    }

    OptionList {
        width: 40%;
        height: 100%;
        border: tall $surface;
    }

    ThemePreview {
        width: 60%;
        height: 100%;
        border: tall $surface;
        padding: 1 2;
        background: $surface;
    }

    #status {
        height: 3;
        border: heavy $surface;
        padding: 0 1;
        margin-top: 1;
    }
    """

    BINDINGS = [
        Binding("r", "refresh", "Refresh list"),
        Binding("q", "quit", "Quit"),
    ]

    status = reactive("Select a theme and press Enter to apply.")

    def __init__(self) -> None:
        super().__init__()
        self.theme_dirs: list[Path] = discover_theme_dirs()
        self.active_dir: Path | None = self.theme_dirs[0] if self.theme_dirs else None

    def compose(self) -> ComposeResult:
        yield Header(show_clock=True)
        with Static(id="container"):
            path_text = (
                f"Themes directory: {self.active_dir}"
                if self.active_dir
                else "No theme directory found. Set THEMES_DIR or run 'make install'."
            )
            yield Static(path_text, id="path")
            
            with Horizontal(id="main-content"):
                self.option_list = OptionList()
                yield self.option_list
                self.preview = ThemePreview()
                yield self.preview
                
            yield Static("Select a theme and press Enter to apply.", id="status")
        yield Footer()

    def on_mount(self) -> None:
        self._reload_options()
        self.option_list.focus()

    def watch_status(self, status: str) -> None:
        with contextlib.suppress(Exception):
            self.query_one("#status", Static).update(status)

    def _reload_options(self) -> None:
        self.option_list.clear_options()
        names = list_themes(self.active_dir)
        for name in names:
            self.option_list.add_option(Option(name, id=name))
        if not names:
            self.status = "No themes available. Run 'make install' to populate themes."
        else:
            # Select first option by default
            self.option_list.action_first()

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

    @work(exclusive=True, thread=True)
    def _apply_theme_task(self, theme: str) -> str:
        """Apply theme in a background thread."""
        cli = find_theme_cli()
        if not cli:
            return "ERROR: cosmikase-theme not found."
            
        result = subprocess.run([cli, theme], capture_output=True, text=True)
        if result.returncode == 0:
            return f"SUCCESS: Applied '{theme}'."
        else:
            detail = result.stderr.strip() or result.stdout.strip() or str(result.returncode)
            return f"ERROR: Failed to apply '{theme}': {detail}"

    def on_worker_state_changed(self, event: Worker.StateChanged) -> None:
        """Handle worker state changes to update status."""
        if event.state == WorkerState.SUCCESS:
            self.status = str(event.worker.result)
        elif event.state == WorkerState.ERROR:
            self.status = f"CRITICAL ERROR: {event.worker.error}"

    @on(OptionList.OptionHighlighted)
    def handle_option_highlighted(self, event: OptionList.OptionHighlighted) -> None:
        if self.active_dir and event.option:
            theme_path = self.active_dir / event.option.prompt
            self.preview.update_preview(theme_path)

    @on(OptionList.OptionSelected)
    def handle_option_selected(self, event: OptionList.OptionSelected) -> None:
        event.stop()
        theme = event.option.prompt
        self.status = f"Applying '{theme}'..."
        self._apply_theme_task(theme)


def run() -> None:
    ThemeTui().run()


if __name__ == "__main__":
    run()
