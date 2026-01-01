# Development Guide

Guide for developers contributing to cosmikase.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Project Structure](#project-structure)
- [Adding Features](#adding-features)
- [Testing](#testing)
- [Code Style](#code-style)
- [Contributing](#contributing)

---

## Getting Started

### Prerequisites

- Pop!_OS 24.04 (or Ubuntu 24.04)
- Git
- Python 3.10+
- `uv` (Python package manager)
- Basic familiarity with:
  - YAML
  - Ansible
  - Bash scripting
  - Python

### Initial Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/paulbrav/omarchy-for-popos.git cosmikase
   cd cosmikase
   ```

2. **Set up development environment:**
   ```bash
   make setup
   ```

   This installs:
   - Python dependencies (via `uv sync`)
   - Ansible and collections
   - Chezmoi (if not present)

3. **Verify setup:**
   ```bash
   # Check Python tools
   uv run cosmikase-config --help
   
   # Check Ansible
   uv run ansible-playbook --version
   
   # Check chezmoi
   chezmoi --version
   ```

---

## Development Environment

### Python Environment

The project uses `uv` for Python dependency management.

**Virtual Environment:**
- Created automatically by `uv sync`
- Located at `.venv/` (gitignored)
- Activated automatically when using `uv run`

**Installing Dependencies:**
```bash
# Install all dependencies
uv sync

# Install with dev extras
uv sync --all-extras

# Add a new dependency
uv add <package-name>

# Add a dev dependency
uv add --dev <package-name>
```

**Running Python Tools:**
```bash
# Use uv run prefix
uv run cosmikase-config list apt core

# Or activate venv manually
source .venv/bin/activate
cosmikase-config list apt core
```

### Ansible Development

**Running Playbook:**
```bash
# Dry-run (no changes)
make dry-run

# Full installation
make install

# With custom config
make install CONFIG_FILE=/path/to/config.yaml
```

**Testing Specific Roles:**
```bash
cd ansible
uv run ansible-playbook -i inventory.yml playbook.yml --tags packages
```

**Verbose Output:**
```bash
cd ansible
uv run ansible-playbook -i inventory.yml playbook.yml -vvv
```

### Chezmoi Development

**Testing Templates:**
```bash
# Initialize chezmoi (if not done)
chezmoi init --source ./chezmoi

# See what would change
chezmoi diff

# Apply changes
chezmoi apply

# Edit a template
chezmoi edit ~/.config/zellij/config.kdl
```

**Template Testing:**
```bash
# Test template rendering
chezmoi execute-template < chezmoi/dot_config/Cursor/User/settings.json.tmpl
```

---

## Project Structure

```
cosmikase/
├── ansible/              # Ansible playbook and roles
│   ├── playbook.yml     # Main playbook
│   ├── inventory.yml    # Inventory (localhost)
│   ├── ansible.cfg      # Ansible configuration
│   └── roles/           # Ansible roles
│       ├── packages/     # Package installation
│       ├── runtimes/     # Runtime installation
│       ├── ghostty/      # Ghostty build
│       ├── tools/        # AI tools installation
│       └── dotfiles/     # Dotfile management
├── bin/                  # Shell scripts
│   ├── cosmikase       # Main menu
│   ├── cosmikase-theme # Theme switching
│   └── ...
├── chezmoi/              # Chezmoi source directory
│   └── dot_config/       # Dotfile templates
├── docs/                 # Documentation
├── src/                  # Python source code
│   └── cosmikase/      # Python package
│       ├── config.py     # Config utilities
│       ├── chezmoi.py    # Chezmoi helpers
│       ├── themes.py     # Theme discovery
│       └── ...
├── themes/               # Theme definitions
├── tests/                # Test files
├── pyproject.toml        # Python project config
├── Makefile              # Development commands
└── cosmikase.yaml      # Main configuration
```

---

## Adding Features

### Adding a New Theme

1. **Create theme directory:**
   ```bash
   mkdir -p themes/my-new-theme
   ```

2. **Create theme files:**
   ```bash
   # Required: theme.yaml
   cat > themes/my-new-theme/theme.yaml <<EOF
   name: My New Theme
   variant: dark
   colors:
     background: "#1a1b26"
     foreground: "#c0caf5"
     accent: "#7aa2f7"
     error: "#f7768e"
     warning: "#e0af68"
   cursor:
     theme: "My Theme Extension"
     extension: "publisher.my-theme"
   wallpaper: backgrounds/wallpaper.png
   EOF
   
   # Add application configs
   # - cosmic.ron
   # - cursor.json
   # - ghostty.conf
   # - etc.
   ```

3. **Add wallpaper:**
   ```bash
   mkdir themes/my-new-theme/backgrounds
   cp wallpaper.png themes/my-new-theme/backgrounds/
   ```

4. **Update Cursor template** (if needed):
   ```bash
   # Edit chezmoi/dot_config/Cursor/User/settings.json.tmpl
   # Add mapping for your theme
   ```

5. **Test the theme:**
   ```bash
   cosmikase-theme my-new-theme
   ```

6. **Update config** (optional):
   ```yaml
   themes:
     available:
       - my-new-theme
   ```

**See Also:** [Theme System Documentation](../themes/README.md)

### Adding a New Package

1. **Add to configuration:**
   ```yaml
   apt:
     core:
       - name: my-package
         desc: Description of my package
         install: true
   ```

2. **Test installation:**
   ```bash
   make dry-run  # Preview changes
   make install  # Install
   ```

3. **No code changes needed** - Ansible handles installation automatically.

### Adding a New Shell Script

1. **Create script in `bin/`:**
   ```bash
   #!/usr/bin/env bash
   # my-new-script - Description
   set -euo pipefail
   
   echo "Hello from my script"
   ```

2. **Make executable:**
   ```bash
   chmod +x bin/my-new-script
   ```

3. **Add to Ansible role:**
   Edit `ansible/roles/dotfiles/tasks/main.yml`:
   ```yaml
   - name: Install my-new-script
     ansible.builtin.copy:
       src: "{{ playbook_dir }}/../bin/my-new-script"
       dest: "{{ local_bin }}/my-new-script"
       mode: "0755"
       remote_src: true
   ```

4. **Test:**
   ```bash
   make install
   my-new-script  # Should work
   ```

### Adding a New Python CLI Tool

1. **Create module in `src/cosmikase/`:**
   ```python
   # src/cosmikase/my_tool.py
   """My new CLI tool."""
   
   def _main() -> None:
       import argparse
       parser = argparse.ArgumentParser()
       parser.add_argument("arg")
       args = parser.parse_args()
       print(f"Hello {args.arg}")
   
   if __name__ == "__main__":
       _main()
   ```

2. **Add entry point in `pyproject.toml`:**
   ```toml
   [project.scripts]
   cosmikase-my-tool = "cosmikase.my_tool:_main"
   ```

3. **Install:**
   ```bash
   uv sync
   ```

4. **Test:**
   ```bash
   uv run cosmikase-my-tool test
   ```

5. **Document:** Add to [CLI Reference](cli-reference.md)

### Adding a New Ansible Role

1. **Create role directory:**
   ```bash
   mkdir -p ansible/roles/my-role/tasks
   ```

2. **Create main task file:**
   ```yaml
   # ansible/roles/my-role/tasks/main.yml
   ---
   - name: Do something
     ansible.builtin.debug:
       msg: "Hello from my role"
   ```

3. **Add to playbook:**
   ```yaml
   # ansible/playbook.yml
   roles:
     - role: my-role
       tags: [my-role]
   ```

4. **Test:**
   ```bash
   cd ansible
   uv run ansible-playbook -i inventory.yml playbook.yml --tags my-role
   ```

---

## Testing

### Running Tests

```bash
# Run all tests
make test

# Run Python tests only
uv run pytest tests/ -v

# Run container smoke test
./tests/container-smoke.sh
```

### Writing Tests

**Python Tests:**
```python
# tests/test_my_module.py
import pytest
from cosmikase.my_module import my_function

def test_my_function():
    assert my_function("input") == "expected_output"
```

**Shell Script Tests:**
```bash
# tests/test_my_script.sh
#!/usr/bin/env bash
set -euo pipefail

# Test my script
output=$(bin/my-script --help)
assert_contains "$output" "Usage"
```

### Linting

```bash
# Run all linters
make lint

# Shell scripts
shellcheck bin/*

# Python code
uv run ruff check src/

# Ansible
cd ansible
uv run ansible-lint playbook.yml
```

### Formatting

```bash
# Format Python code
make fmt

# Manual formatting
uv run ruff format src/
uv run ruff check --fix src/
```

---

## Code Style

### Python

**Style Guide:** Follow PEP 8 with these exceptions:
- Line length: 100 characters
- Use type hints for all functions
- Use `snake_case` for functions and variables
- Use `PascalCase` for classes

**Tools:**
- `ruff` for linting and formatting
- `mypy` for type checking (optional)

**Example:**
```python
from pathlib import Path
from typing import Optional

def process_config(config_path: Path) -> Optional[dict]:
    """Process configuration file.
    
    Args:
        config_path: Path to config file.
        
    Returns:
        Parsed config dict or None if error.
    """
    try:
        return yaml.safe_load(config_path.read_text())
    except Exception:
        return None
```

### Shell Scripts

**Style Guide:**
- Use `set -euo pipefail` at the top
- Use `snake_case` for variables
- Quote all variables: `"$VAR"`
- Use `[[ ]]` for conditionals
- Add shebang: `#!/usr/bin/env bash`

**Example:**
```bash
#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

main() {
    local arg="$1"
    if [[ -z "$arg" ]]; then
        echo "Error: argument required" >&2
        exit 1
    fi
    echo "Processing: $arg"
}

main "$@"
```

### Ansible

**Style Guide:**
- Use `ansible.builtin` modules when possible
- Use `become: true` for system-level changes
- Use `when` conditions for optional tasks
- Use `tags` for role organization
- Add descriptive task names

**Example:**
```yaml
- name: Install core packages
  become: true
  ansible.builtin.apt:
    name: "{{ apt.core | map(attribute='name') | list }}"
    state: present
  when: apt.core is defined
  tags: [packages]
```

### YAML

**Style Guide:**
- Use 2-space indentation
- Use `-` for list items
- Quote strings with special characters
- Add comments for clarity

**Example:**
```yaml
defaults:
  install: true
  theme: nord

apt:
  core:
    - name: git
      desc: Version control system
      install: true
```

---

## Contributing

### Workflow

1. **Fork the repository**

2. **Create a branch:**
   ```bash
   git checkout -b feature/my-feature
   ```

3. **Make changes:**
   - Write code
   - Add tests
   - Update documentation
   - Run linters

4. **Commit changes:**
   ```bash
   git add .
   git commit -m "Add my feature"
   ```
   
   **Commit Message Format:**
   - Use imperative mood: "Add feature" not "Added feature"
   - Keep first line under 72 characters
   - Add body if needed for explanation

5. **Push and create PR:**
   ```bash
   git push origin feature/my-feature
   ```
   
   Then create a pull request on GitHub.

### Pull Request Guidelines

**Before Submitting:**
- [ ] Code follows style guidelines
- [ ] Tests pass (`make test`)
- [ ] Linting passes (`make lint`)
- [ ] Documentation updated
- [ ] Commit messages are clear

**PR Description Should Include:**
- What changed and why
- How to test the changes
- Screenshots (if UI changes)
- Related issues

### Code Review

**Reviewers Will Check:**
- Code quality and style
- Test coverage
- Documentation completeness
- Backward compatibility
- Security considerations

**Responding to Feedback:**
- Address all comments
- Ask for clarification if needed
- Update PR as requested
- Keep discussion constructive

---

## Debugging

### Python Tools

```bash
# Run with debug output
uv run python -m cosmikase.config list apt core

# Use Python debugger
uv run python -m pdb -m cosmikase.config list apt core
```

### Ansible

```bash
# Verbose output
cd ansible
uv run ansible-playbook -i inventory.yml playbook.yml -vvv

# Check specific task
uv run ansible-playbook -i inventory.yml playbook.yml --tags packages -vvv

# Dry-run with check
uv run ansible-playbook -i inventory.yml playbook.yml --check
```

### Shell Scripts

```bash
# Enable debug mode
set -x  # Add to script

# Or run with bash -x
bash -x bin/cosmikase-theme nord
```

### Chezmoi

```bash
# Check status
chezmoi status

# See differences
chezmoi diff

# Doctor check
chezmoi doctor -v

# Test template
chezmoi execute-template < chezmoi/dot_config/Cursor/User/settings.json.tmpl
```

---

## Common Tasks

### Updating Dependencies

```bash
# Update Python dependencies
uv sync --upgrade

# Update Ansible collections
uv run ansible-galaxy collection install --upgrade community.general

# Update lock file
uv lock --upgrade
```

### Adding a New Dependency

```bash
# Python package
uv add <package-name>

# Dev dependency
uv add --dev <package-name>

# Ansible collection
# Edit ansible/requirements.yml, then:
uv run ansible-galaxy collection install -r ansible/requirements.yml
```

### Building Documentation

Documentation is in Markdown format. To preview:

```bash
# Use any Markdown viewer
# Or push to GitHub to see rendered version
```

### Release Process

1. Update version in `pyproject.toml`
2. Update `CHANGELOG.md` (if exists)
3. Create git tag
4. Push tag
5. GitHub Actions will handle release (if configured)

---

## Getting Help

### Documentation

- [README.md](../README.md) - Project overview
- [CLI Reference](cli-reference.md) - Command documentation
- [Configuration Reference](configuration-reference.md) - Config schema
- [Architecture Guide](architecture.md) - System design
- [Troubleshooting Guide](troubleshooting.md) - Common issues

### Communication

- **Issues:** GitHub Issues for bugs and feature requests
- **Discussions:** GitHub Discussions for questions
- **Pull Requests:** For code contributions

### Resources

- [Ansible Documentation](https://docs.ansible.com/)
- [Chezmoi Documentation](https://www.chezmoi.io/)
- [Python Packaging Guide](https://packaging.python.org/)
- [Shell Scripting Best Practices](https://google.github.io/styleguide/shellguide.html)

---

## See Also

- [Architecture Guide](architecture.md) - System design
- [CLI Reference](cli-reference.md) - Command documentation
- [Troubleshooting Guide](troubleshooting.md) - Debugging help

