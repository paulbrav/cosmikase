"""Pydantic models for cosmikase configuration validation.

This module defines the schema for cosmikase.yaml configuration files,
enabling validation and providing better error messages.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any, Literal

import yaml
from pydantic import BaseModel, Field, field_validator


class PackageItem(BaseModel):
    """APT package configuration."""

    name: str
    desc: str | None = None
    install: bool = True
    alias: str | None = None
    source: str | None = None
    note: str | None = None


class FlatpakItem(BaseModel):
    """Flatpak application configuration."""

    id: str = Field(..., description="Flatpak application ID (e.g., org.example.App)")
    desc: str | None = None
    install: bool = True


class FontItem(BaseModel):
    """Font configuration."""

    name: str
    desc: str | None = None
    url: str
    install: bool = True


class InstallerItem(BaseModel):
    """Custom installer configuration."""

    name: str
    desc: str | None = None
    method: Literal[
        "script",
        "deb",
        "npm",
        "bun",
        "tarball",
        "custom_nvm",
        "custom_antigravity",
        "custom_brave",
        "custom_dangerzone",
        "manual",
    ]
    url: str | None = None
    deb_url: str | None = None
    npm_package: str | None = None
    bun_package: str | None = None
    args: str | None = None
    check: str | None = None
    note: str | None = None
    install: bool = True


class NpmItem(BaseModel):
    """NPM package configuration."""

    name: str
    desc: str | None = None
    version: str = "latest"
    install: bool = True


class UvToolItem(BaseModel):
    """UV tool configuration."""

    name: str
    desc: str | None = None
    install: bool = True


class DefaultsConfig(BaseModel):
    """Default settings."""

    install: bool = True
    ghostty: bool = True
    yubikey_setup: bool = False
    theme: str = "nord"
    run_fw_update: bool = True
    run_recovery_upgrade: bool = False


class AptConfig(BaseModel):
    """APT packages configuration."""

    core: list[PackageItem] = Field(default_factory=list)
    yubikey: list[PackageItem] = Field(default_factory=list)
    gui: list[PackageItem] = Field(default_factory=list)
    terminal: list[PackageItem] = Field(default_factory=list)


class FlatpakConfig(BaseModel):
    """Flatpak applications configuration."""

    utility: list[FlatpakItem] = Field(default_factory=list)


class FontsConfig(BaseModel):
    """Fonts configuration."""

    nerd: list[FontItem] = Field(default_factory=list)


class WebItem(BaseModel):
    """Web application configuration."""

    name: str
    desc: str | None = None
    url: str
    icon_url: str | None = None
    install: bool = True


class WebConfig(BaseModel):
    """Web applications configuration."""

    apps: list[WebItem] = Field(default_factory=list)


class InstallersConfig(BaseModel):
    """Custom installers configuration."""

    runtimes: list[InstallerItem] = Field(default_factory=list)
    ai_tools: list[InstallerItem] = Field(default_factory=list)
    security: list[InstallerItem] = Field(default_factory=list)


class ThemesConfig(BaseModel):
    """Themes configuration."""

    default: str = "nord"
    available: list[str] = Field(default_factory=list)
    paths: dict[str, str] = Field(default_factory=dict)


class HardwareConfig(BaseModel):
    """Hardware-specific configuration."""

    emit_notes: bool = True
    oem_kernel: str | None = None
    warn_on_mix: bool = True
    notes: str | None = None


class CosmikaseConfig(BaseModel):
    """Root configuration model for cosmikase.yaml."""

    defaults: DefaultsConfig = Field(default_factory=DefaultsConfig)
    apt: AptConfig = Field(default_factory=AptConfig)
    flatpak: FlatpakConfig = Field(default_factory=FlatpakConfig)
    fonts: FontsConfig = Field(default_factory=FontsConfig)
    web: WebConfig = Field(default_factory=WebConfig)
    installers: InstallersConfig = Field(default_factory=InstallersConfig)
    npm: list[NpmItem | str] = Field(default_factory=list)
    uv_tools: list[UvToolItem | str] = Field(default_factory=list)
    themes: ThemesConfig = Field(default_factory=ThemesConfig)
    scripts: list[Any] = Field(default_factory=list)
    hp_zbook_ultra: HardwareConfig | None = None

    @field_validator("npm", "uv_tools", mode="before")
    @classmethod
    def normalize_string_items(cls, v: list) -> list:
        """Convert string items to dict format."""
        if not isinstance(v, list):
            return v
        result = []
        for item in v:
            if isinstance(item, str):
                result.append({"name": item})
            else:
                result.append(item)
        return result


def load_and_validate(path: Path | str) -> CosmikaseConfig:
    """Load and validate a cosmikase.yaml configuration file.

    Args:
        path: Path to the configuration file.

    Returns:
        Validated CosmikaseConfig object.

    Raises:
        pydantic.ValidationError: If the configuration is invalid.
        FileNotFoundError: If the file doesn't exist.
    """
    path = Path(path)
    with open(path) as f:
        data = yaml.safe_load(f)
    return CosmikaseConfig.model_validate(data)


def validate_config(path: Path | str) -> tuple[bool, list[str]]:
    """Validate a configuration file and return errors if any.

    Args:
        path: Path to the configuration file.

    Returns:
        Tuple of (is_valid, list_of_error_messages)
    """
    try:
        load_and_validate(path)
        return True, []
    except Exception as e:
        return False, [str(e)]


def _main() -> None:
    """CLI entry point for configuration validation."""
    import argparse
    import sys

    parser = argparse.ArgumentParser(description="Validate cosmikase configuration")
    parser.add_argument(
        "config",
        nargs="?",
        default="cosmikase.yaml",
        help="Path to config file (default: cosmikase.yaml)",
    )
    parser.add_argument(
        "--quiet",
        "-q",
        action="store_true",
        help="Only print errors",
    )

    args = parser.parse_args()

    is_valid, errors = validate_config(args.config)

    if is_valid:
        if not args.quiet:
            print(f"✓ Configuration is valid: {args.config}")
        sys.exit(0)
    else:
        print(f"✗ Configuration errors in {args.config}:", file=sys.stderr)
        for error in errors:
            print(f"  {error}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    _main()
