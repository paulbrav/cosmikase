# Troubleshooting Guide

Common issues and solutions for omarchy-for-popos.

## Table of Contents

- [Installation Issues](#installation-issues)
- [Theme Switching Problems](#theme-switching-problems)
- [CLI Command Errors](#cli-command-errors)
- [Ansible Failures](#ansible-failures)
- [Chezmoi Issues](#chezmoi-issues)
- [Configuration Problems](#configuration-problems)
- [Recovery Procedures](#recovery-procedures)
- [Debug Commands](#debug-commands)

---

## Installation Issues

### uv Not Found

**Symptom:**
```
Error: uv is not installed
```

**Solution:**
```bash
# Install uv
curl -LsSf https://astral.sh/uv/install.sh | sh

# Add to PATH (if not already)
export PATH="$HOME/.local/bin:$PATH"

# Verify
which uv
```

**Prevention:** Run `make setup` before `make install` to ensure all dependencies are installed.

---

### Chezmoi Not Found

**Symptom:**
```
Error: chezmoi not found
```

**Solution:**
```bash
# Install chezmoi
curl -sfL https://get.chezmoi.io | sh

# Or via make setup
make setup
```

**Verify:**
```bash
which chezmoi
chezmoi --version
```

---

### Ansible Collection Missing

**Symptom:**
```
ERROR! couldn't resolve module/action 'ansible.posix.synchronize'
```

**Solution:**
```bash
# Install required collections
uv run ansible-galaxy collection install community.general ansible.posix

# Or run make setup which does this automatically
make setup
```

---

### Permission Denied Errors

**Symptom:**
```
Permission denied: /home/user/.local/bin/omarchy-pop-theme
```

**Solution:**
```bash
# Make scripts executable
chmod +x ~/.local/bin/omarchy-pop-*

# Or re-run installation
make install
```

---

### Ghostty Build Fails

**Symptom:**
```
Warning: Ghostty build failed
```

**Causes:**
- Zig 0.13+ not installed
- Missing build dependencies
- Network issues during git pull

**Solution:**
```bash
# Check Zig version
zig version  # Should be 0.13.0 or higher

# Install Zig if missing
# See: https://ziglang.org/download/

# Check Ghostty source directory
ls -la ~/ghostty-source

# Try manual build
cd ~/ghostty-source
git pull
zig build -Doptimize=ReleaseFast
```

**Workaround:** Disable Ghostty build in config:
```yaml
defaults:
  ghostty: false
```

---

## Theme Switching Problems

### Theme Not Found

**Symptom:**
```
Error: Theme 'xyz' not found
```

**Solution:**
```bash
# List available themes
omarchy-themes-dir --list

# Or check themes directory
ls -la ~/.local/share/omarchy-pop/themes/

# Verify theme exists
ls -la ~/.local/share/omarchy-pop/themes/nord/
```

**Check theme name:** Theme names are case-sensitive and must match directory names exactly.

---

### Cursor Theme Not Applying

**Symptom:** Theme name is set but colors don't change.

**Solutions:**
1. **Reload Cursor window:**
   - Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on macOS)
   - Type "reload"
   - Select "Developer: Reload Window"

2. **Verify extension is installed:**
   ```bash
   # Check installed extensions
   cursor --list-extensions | grep -i catppuccin
   
   # Install if missing
   omarchy-cursor-extensions install
   ```

3. **Check settings.json:**
   ```bash
   cat ~/.config/Cursor/User/settings.json | grep colorTheme
   ```

4. **Verify theme name matches exactly:**
   - Theme name in settings must match VS Code extension name exactly
   - Check [editor-theming.md](editor-theming.md) for correct names

**See Also:** [Editor Theming Guide](editor-theming.md)

---

### COSMIC Theme Not Updating

**Symptom:** Desktop theme doesn't change after `omarchy-pop-theme`.

**Solutions:**
1. **Check COSMIC settings daemon:**
   ```bash
   # Restart COSMIC settings daemon
   systemctl --user restart cosmic-settings-daemon
   ```

2. **Verify RON file was updated:**
   ```bash
   cat ~/.config/cosmic/com.system76.CosmicTheme.Mode/v1/is_dark
   ```

3. **Check theme file exists:**
   ```bash
   ls -la ~/.local/share/omarchy-pop/themes/nord/cosmic.ron
   ```

4. **Log out and back in** (sometimes required for full theme application)

**See Also:** [COSMIC Theming Guide](cosmic-theming.md)

---

### Chezmoi Conflicts During Theme Switch

**Symptom:**
```
chezmoi: source file conflicts with target
```

**Solution:**
```bash
# See what would change
chezmoi diff

# Force apply (overwrites local changes)
chezmoi apply --force

# Or merge conflicts manually
chezmoi merge <file>
```

**Prevention:** Don't manually edit files managed by chezmoi. Edit templates in `chezmoi/` instead.

---

### Theme History Missing (Rollback Fails)

**Symptom:**
```
Error: No theme history found to roll back
```

**Solution:**
```bash
# Check history file
cat ~/.local/share/omarchy-pop/theme-history.txt

# Manually switch to previous theme
omarchy-pop-theme <previous-theme-name>
```

**Note:** History is created automatically on first theme switch. If missing, rollback won't work.

---

## CLI Command Errors

### Command Not Found

**Symptom:**
```
omarchy-pop-theme: command not found
```

**Solutions:**
1. **Check PATH:**
   ```bash
   echo $PATH | grep -q "$HOME/.local/bin" || export PATH="$HOME/.local/bin:$PATH"
   ```

2. **Verify installation:**
   ```bash
   ls -la ~/.local/bin/omarchy-pop-*
   ```

3. **Re-run installation:**
   ```bash
   make install
   ```

4. **Use uv run:**
   ```bash
   uv run omarchy-config list apt core
   ```

---

### omarchy-config Fails

**Symptom:**
```
Config file not found: omarchy-pop.yaml
```

**Solutions:**
```bash
# Run from repository root
cd ~/Repos/omarchy-for-popos
omarchy-config list apt core

# Or specify config path
omarchy-config --config /path/to/omarchy-pop.yaml list apt core

# Or set environment variable
export OMARCHY_POP_CONFIG=/path/to/config.yaml
omarchy-config list apt core
```

---

### gum Not Found

**Symptom:**
```
Error: gum is not installed
```

**Solution:**
```bash
# Install gum
sudo apt install gum

# Or via omarchy-pop (if already partially installed)
omarchy-pop-install  # Select gum from optional software
```

---

### Python CLI Tools Not Found

**Symptom:**
```
omarchy-config: command not found
```

**Solutions:**
1. **Install Python tools:**
   ```bash
   make setup
   ```

2. **Use uv run:**
   ```bash
   uv run omarchy-config list apt core
   ```

3. **Check installation:**
   ```bash
   uv run python -m omarchy_pop.config --help
   ```

---

## Ansible Failures

### Playbook Fails with "No such file or directory"

**Symptom:**
```
ERROR! the file '/path/to/omarchy-pop.yaml' was not found
```

**Solution:**
```bash
# Run from repository root
cd ~/Repos/omarchy-for-popos

# Or specify config file
make install CONFIG_FILE=/path/to/omarchy-pop.yaml
```

---

### APT Package Installation Fails

**Symptom:**
```
Failed to install package: <package-name>
```

**Solutions:**
1. **Update package cache:**
   ```bash
   sudo apt update
   ```

2. **Check package exists:**
   ```bash
   apt search <package-name>
   ```

3. **Try installing manually:**
   ```bash
   sudo apt install <package-name>
   ```

4. **Check for typos in config:**
   ```bash
   omarchy-config list apt core --names-only
   ```

---

### Flatpak Installation Fails

**Symptom:**
```
Error: Unable to install <app-id>
```

**Solutions:**
1. **Check Flatpak remote:**
   ```bash
   flatpak remote-list
   ```

2. **Add Flathub if missing:**
   ```bash
   flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
   ```

3. **Update remotes:**
   ```bash
   flatpak update --appstream
   ```

4. **Verify app ID:**
   ```bash
   flatpak search <app-name>
   ```

---

### Ghostty Build Fails in Ansible

**Symptom:**
```
fatal: [localhost]: FAILED! => {"msg": "Ghostty build failed"}
```

**Solutions:**
1. **Check Zig installation:**
   ```bash
   which zig
   zig version  # Must be 0.13.0+
   ```

2. **Check Ghostty source:**
   ```bash
   ls -la ~/ghostty-source
   ```

3. **Disable Ghostty build:**
   ```yaml
   defaults:
     ghostty: false
   ```

4. **Build manually:**
   ```bash
   cd ~/ghostty-source
   git pull
   zig build -Doptimize=ReleaseFast
   ```

---

## Chezmoi Issues

### Chezmoi Apply Fails

**Symptom:**
```
chezmoi apply: error applying dotfiles
```

**Solutions:**
1. **Check chezmoi status:**
   ```bash
   chezmoi status
   ```

2. **See what would change:**
   ```bash
   chezmoi diff
   ```

3. **Force apply:**
   ```bash
   chezmoi apply --force
   ```

4. **Check for syntax errors in templates:**
   ```bash
   chezmoi doctor
   ```

---

### Chezmoi Config Corrupted

**Symptom:**
```
Error: chezmoi.toml has invalid TOML syntax
```

**Solution:**
```bash
# Backup current config
cp ~/.config/chezmoi/chezmoi.toml ~/.config/chezmoi/chezmoi.toml.bak

# Regenerate config
omarchy-chezmoi nord ~/.local/share/omarchy-pop/themes

# Or edit manually
nano ~/.config/chezmoi/chezmoi.toml
```

**Validate TOML:**
```bash
python3 -c "import tomli; tomli.load(open('~/.config/chezmoi/chezmoi.toml', 'rb'))"
```

---

### Dotfiles Not Updating

**Symptom:** Changes to templates don't appear in dotfiles.

**Solutions:**
1. **Re-apply chezmoi:**
   ```bash
   chezmoi apply
   ```

2. **Check template syntax:**
   ```bash
   chezmoi doctor
   ```

3. **Verify template was modified:**
   ```bash
   chezmoi diff
   ```

4. **Force re-apply:**
   ```bash
   chezmoi apply --force
   ```

---

## Configuration Problems

### Invalid YAML Syntax

**Symptom:**
```
ERROR! YAML syntax error in omarchy-pop.yaml
```

**Solution:**
```bash
# Validate YAML
python3 -c "import yaml; yaml.safe_load(open('omarchy-pop.yaml'))"

# Or use online validator
# https://www.yamllint.com/
```

**Common Issues:**
- Missing colons after keys
- Incorrect indentation (must use spaces, not tabs)
- Unquoted strings with special characters
- Missing quotes around values containing colons

---

### Section Not Found Error

**Symptom:**
```
Error: Section 'xyz' not found in config
```

**Solution:**
```bash
# List available sections
omarchy-config list apt  # Will show error with available sections

# Check config structure
cat omarchy-pop.yaml | grep -E "^[a-z_]+:"
```

**Common Mistakes:**
- Typo in section name (case-sensitive)
- Missing section in config file
- Incorrect indentation

---

### Package Not Installing

**Symptom:** Package listed in config but not installed.

**Solutions:**
1. **Check install flag:**
   ```bash
   omarchy-config list apt core --all | grep <package-name>
   ```

2. **Verify defaults.install:**
   ```bash
   omarchy-config get defaults.install
   ```

3. **Check if package was skipped:**
   ```bash
   # Re-run installation
   make install
   ```

4. **Install manually:**
   ```bash
   sudo apt install <package-name>
   ```

---

## Recovery Procedures

### Rollback Theme Change

**Solution:**
```bash
# Use rollback if available
omarchy-pop-theme --rollback

# Or manually switch to previous theme
omarchy-pop-theme <previous-theme-name>

# Check theme history
cat ~/.local/share/omarchy-pop/theme-history.txt
```

---

### Purge Chezmoi

**Warning:** This removes all chezmoi-managed dotfiles.

**Solution:**
```bash
# See what would be removed
chezmoi diff

# Remove chezmoi-managed files
chezmoi purge

# Re-initialize
chezmoi init --source ~/Repos/omarchy-for-popos/chezmoi
chezmoi apply
```

---

### Reset Configuration

**Solution:**
```bash
# Backup current config
cp omarchy-pop.yaml omarchy-pop.yaml.bak

# Restore from repository
git checkout omarchy-pop.yaml

# Or start fresh
cp omarchy-pop.yaml.example omarchy-pop.yaml
```

---

### Uninstall Everything

**Solution:**
```bash
# Remove dotfiles
chezmoi purge

# Remove scripts
rm -rf ~/.local/bin/omarchy-pop-*

# Remove themes
rm -rf ~/.local/share/omarchy-pop

# Remove shell integration (edit manually)
# Remove OMARCHY-POP MANAGED BLOCK from ~/.bashrc and ~/.zshrc

# Remove chezmoi config
rm -rf ~/.config/chezmoi
```

**Note:** This does not uninstall packages. Remove those manually:
```bash
# List installed packages
omarchy-config list apt core --names-only | xargs sudo apt remove

# Remove Flatpak apps
omarchy-config list flatpak utility --names-only | xargs flatpak uninstall
```

---

## Debug Commands

### Check Installation Status

```bash
# Verify scripts are installed
ls -la ~/.local/bin/omarchy-pop-*

# Check Python tools
uv run omarchy-config --help

# Verify themes directory
omarchy-themes-dir --list

# Check chezmoi status
chezmoi status
```

---

### Validate Configuration

```bash
# Validate YAML syntax
python3 -c "import yaml; yaml.safe_load(open('omarchy-pop.yaml'))"

# Check config structure
omarchy-config get defaults.theme

# List all sections
omarchy-config list apt
```

---

### Check Chezmoi

```bash
# Check chezmoi health
chezmoi doctor

# See what would change
chezmoi diff

# Check config
cat ~/.config/chezmoi/chezmoi.toml

# Verify templates
chezmoi execute-template < ~/.local/share/chezmoi/dot_config/shell/omarchy-pop.sh.tmpl
```

---

### Debug Theme Switching

```bash
# Check current theme
omarchy-config get defaults.theme

# List available themes
omarchy-themes-dir --list

# Check theme files
ls -la ~/.local/share/omarchy-pop/themes/nord/

# Verify chezmoi data
cat ~/.config/chezmoi/chezmoi.toml | grep theme

# Check theme history
cat ~/.local/share/omarchy-pop/theme-history.txt
```

---

### Debug Ansible

```bash
# Dry-run (no changes)
make dry-run

# Verbose output
cd ansible
uv run ansible-playbook -i inventory.yml playbook.yml -vvv

# Check specific role
uv run ansible-playbook -i inventory.yml playbook.yml --tags packages

# Test config loading
uv run ansible-playbook -i inventory.yml playbook.yml --check
```

---

### Check Logs

```bash
# System logs (for udev/power helper)
journalctl -t omarchy-power

# Ansible logs (if redirected)
# Check output from make install

# Chezmoi logs (if enabled)
chezmoi doctor -v
```

---

## Getting Help

### Before Asking for Help

1. **Check documentation:**
   - [README.md](../README.md)
   - [CLI Reference](cli-reference.md)
   - [Configuration Reference](configuration-reference.md)

2. **Run debug commands:**
   - See [Debug Commands](#debug-commands) section above

3. **Check for similar issues:**
   - Search repository issues
   - Check recent commits

### Providing Debug Information

When reporting issues, include:

```bash
# System information
uname -a
lsb_release -a

# Installation status
which omarchy-pop-theme
omarchy-config --version 2>/dev/null || uv run omarchy-config --help

# Configuration
omarchy-config get defaults.theme
omarchy-config get defaults.install

# Chezmoi status
chezmoi doctor

# Error messages
# Full output of failing command
```

---

## Common Workarounds

### Use uv run for Python Tools

If Python CLI tools aren't on PATH:

```bash
uv run omarchy-config list apt core
uv run theme-tui
```

### Manual Theme Application

If `omarchy-pop-theme` fails:

```bash
# Update chezmoi manually
omarchy-chezmoi nord ~/.local/share/omarchy-pop/themes

# Apply dotfiles
chezmoi apply

# Reload applications manually
# Cursor: Ctrl+Shift+P -> "Developer: Reload Window"
# COSMIC: Log out and back in
```

### Skip Problematic Steps

Use flags to skip failing steps:

```bash
# Skip Cursor update
omarchy-pop-theme nord --no-cursor

# Skip COSMIC update
omarchy-pop-theme nord --no-cosmic

# Skip chezmoi (only update running apps)
omarchy-pop-theme nord --no-chezmoi
```

---

## See Also

- [CLI Reference](cli-reference.md) - Complete command documentation
- [Configuration Reference](configuration-reference.md) - Config file schema
- [Editor Theming Guide](editor-theming.md) - Theme troubleshooting
- [COSMIC Theming Guide](cosmic-theming.md) - Desktop theme issues

