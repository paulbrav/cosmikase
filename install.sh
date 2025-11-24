#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_FILE="${CONFIG_FILE:-$REPO_DIR/omarchy-pop.yaml}"
TARGET_BIN="$HOME/.local/bin"
TARGET_THEMES="$HOME/.local/share/omarchy-pop/themes"
TARGET_CONFIG="$HOME/.config"
SUDO="sudo"
APT_CACHE_READY=false
if [[ "$(id -u)" -eq 0 ]]; then
  SUDO=""
elif ! command -v sudo >/dev/null 2>&1; then
  SUDO=""
fi

error() { echo "[omarchy-pop] $*" >&2; }

die() { error "$*"; exit 1; }

need_file() { [[ -f "$1" ]] || die "Missing config file: $1"; }

ensure_pyyaml() {
  if ! python3 -c "import yaml" 2>/dev/null; then
    echo "Installing python3-yaml (requires sudo)..."
    $SUDO apt update
    $SUDO apt install -y python3-yaml
  fi
}

yaml_list() {
  local section="$1" group="$2"
  python3 - <<'PY' "$CONFIG_FILE" "$section" "$group"
import sys, yaml
cfg = yaml.safe_load(open(sys.argv[1]))
section, group = sys.argv[2], sys.argv[3]
for item in cfg.get(section, {}).get(group, []):
    if item.get('install', True):
        name = item.get('name') or item.get('id')
        extras = []
        for key in ('source','alias','url'):
            if key in item:
                extras.append(f"{key}={item[key]}")
        print(name + ("|" + ",".join(extras) if extras else ""))
PY
}

yaml_top_list() {
  local section="$1"
  python3 - <<'PY' "$CONFIG_FILE" "$section"
import sys, yaml
cfg = yaml.safe_load(open(sys.argv[1]))
section = sys.argv[2]
items = cfg.get(section, [])
if isinstance(items, list):
    for item in items:
        if isinstance(item, dict):
            if item.get('install', True):
                print(item.get('name'))
        else:
            print(item)
PY
}

yaml_installers() {
  local section="$1" group="$2"
  python3 - <<'PY' "$CONFIG_FILE" "$section" "$group"
import sys, yaml
cfg = yaml.safe_load(open(sys.argv[1]))
section, group = sys.argv[2], sys.argv[3]
for item in cfg.get(section, {}).get(group, []):
    if not item.get('install', True):
        continue
    name = item.get('name') or item.get('id')
    fields = []
    for key in ('method','url','args','check','id','deb_url','note','npm_package','bun_package'):
        if key in item and item.get(key) is not None:
            fields.append(f"{key}={item[key]}")
    print(name + ("|" + ",".join(fields) if fields else ""))
PY
}

yaml_value() {
  local path="$1" default="$2"
  python3 - <<'PY' "$CONFIG_FILE" "$path" "$default"
import sys, yaml
cfg = yaml.safe_load(open(sys.argv[1]))
path = sys.argv[2]
default = sys.argv[3]
cur = cfg
for part in path.split('.'):
    if isinstance(cur, dict):
        cur = cur.get(part)
    else:
        cur = None
        break
if cur is None:
    cur = default
if isinstance(cur, bool):
    print('true' if cur else 'false')
else:
    print(cur)
PY
}

have_pkg() { dpkg -s "$1" &>/dev/null; }

install_apt_list() {
  local label="$1"; shift
  local pkgs=("$@")
  if [[ "$APT_CACHE_READY" != "true" ]]; then
    $SUDO apt update
    APT_CACHE_READY=true
  fi
  local to_install=()
  for pkg in "${pkgs[@]}"; do
    [[ -z "$pkg" || "$pkg" == ghostty ]] && continue
    if ! have_pkg "$pkg"; then
      if ! apt-cache show "$pkg" >/dev/null 2>&1; then
        echo "Skipping $pkg (not found in apt cache)"
        continue
      fi
      to_install+=("$pkg")
    fi
  done
  if ((${#to_install[@]})); then
    echo "Installing APT ($label): ${to_install[*]}"
    $SUDO apt install -y "${to_install[@]}"
  else
    echo "APT ($label): nothing to do"
  fi
}

install_flatpaks() {
  mapfile -t flatpaks < <(yaml_list flatpak utility)
  if ! command -v flatpak >/dev/null 2>&1; then
    echo "Installing flatpak (requires sudo)..."
    if [[ "$APT_CACHE_READY" != "true" ]]; then
      $SUDO apt update
      APT_CACHE_READY=true
    fi
    $SUDO apt install -y flatpak
  fi
  if ! flatpak remotes | grep -q flathub; then
    $SUDO flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
  fi
  for app in "${flatpaks[@]}"; do
    [[ -z "$app" ]] && continue
    if ! flatpak list | awk '{print $1}' | grep -qx "$app"; then
      echo "Installing Flatpak $app"
      flatpak install -y flathub "$app"
    else
      echo "Flatpak $app already installed"
    fi
  done
}

install_ghostty() {
  local url="${GHOSTTY_DEB_URL:-https://github.com/ghostty-org/ghostty/releases/latest/download/ghostty_amd64.deb}"
  if have_pkg ghostty; then
    echo "Ghostty already installed"
    return 0
  fi
  echo "Installing Ghostty from $url"
  tmpdir=$(mktemp -d)
  trap 'rm -rf "$tmpdir"' EXIT
  
  if ! curl -fsSLo "$tmpdir/ghostty.deb" "$url"; then
      echo "Error: Failed to download Ghostty from $url"
      rm -rf "$tmpdir"
      trap - EXIT
      return 1
  fi

  if ! $SUDO dpkg -i "$tmpdir/ghostty.deb"; then
      echo "dpkg error encountered; attempting fix with 'apt -f install'..."
      if ! $SUDO apt -f install -y; then
          echo "Error: Failed to install Ghostty dependencies"
          rm -rf "$tmpdir"
          trap - EXIT
          return 1
      fi
  fi
  rm -rf "$tmpdir"
  trap - EXIT
}

install_dangerzone() {
  if have_pkg dangerzone; then
    echo "Dangerzone already installed"
    return 0
  fi
  echo "Installing Dangerzone..."
  
  if [[ "$APT_CACHE_READY" != "true" ]]; then
    $SUDO apt update
    APT_CACHE_READY=true
  fi
  $SUDO apt install -y gpg ca-certificates

  # Keyring setup
  if [[ ! -f /etc/apt/keyrings/fpf-apt-tools-archive-keyring.gpg ]]; then
      echo "Setting up Dangerzone GPG key..."
      $SUDO mkdir -p /etc/apt/keyrings
      
      local tmp_home
      tmp_home=$(mktemp -d)
      chmod 700 "$tmp_home"
      
      if ! $SUDO gpg --keyserver hkps://keys.openpgp.org \
        --no-default-keyring --no-permission-warning --homedir "$tmp_home" \
        --keyring gnupg-ring:/etc/apt/keyrings/fpf-apt-tools-archive-keyring.gpg \
        --recv-keys DE28AB241FA48260FAC9B8BAA7C9B38522604281; then
         echo "Failed to receive GPG key"
         rm -rf "$tmp_home"
         return 1
      fi
      rm -rf "$tmp_home"
      $SUDO chmod +r /etc/apt/keyrings/fpf-apt-tools-archive-keyring.gpg
  fi
  
  # Repo source
  if [[ ! -f /etc/apt/sources.list.d/fpf-apt-tools.list ]]; then
      echo "Adding Dangerzone repository..."
      . /etc/os-release
      local code="${VERSION_CODENAME:-jammy}"
      echo "deb [signed-by=/etc/apt/keyrings/fpf-apt-tools-archive-keyring.gpg] \
        https://packages.freedom.press/apt-tools-prod $code main" \
        | $SUDO tee /etc/apt/sources.list.d/fpf-apt-tools.list > /dev/null
      
      $SUDO apt update
      APT_CACHE_READY=true
  fi
  
  $SUDO apt install -y dangerzone
}

install_antigravity() {
  if have_pkg antigravity; then
    echo "Antigravity already installed"
    return 0
  fi
  echo "Installing Antigravity..."

  $SUDO mkdir -p /etc/apt/keyrings
  
  if [[ ! -f /etc/apt/keyrings/antigravity-repo-key.gpg ]]; then
      echo "Setting up Antigravity GPG key..."
      if ! curl -fsSL https://us-central1-apt.pkg.dev/doc/repo-signing-key.gpg | \
           $SUDO gpg --dearmor -o /etc/apt/keyrings/antigravity-repo-key.gpg; then
          echo "Failed to download/install Antigravity GPG key"
          return 1
      fi
  fi

  if [[ ! -f /etc/apt/sources.list.d/antigravity.list ]]; then
      echo "Adding Antigravity repository..."
      echo "deb [signed-by=/etc/apt/keyrings/antigravity-repo-key.gpg] https://us-central1-apt.pkg.dev/projects/antigravity-auto-updater-dev/ antigravity-debian main" | \
        $SUDO tee /etc/apt/sources.list.d/antigravity.list > /dev/null
      
      $SUDO apt update
      APT_CACHE_READY=true
  fi

  $SUDO apt install -y antigravity
}

install_fonts() {
  if ! command -v unzip >/dev/null 2>&1; then
    echo "Installing unzip (required for fonts)..."
    if [[ "$APT_CACHE_READY" != "true" ]]; then
      $SUDO apt update
      APT_CACHE_READY=true
    fi
    $SUDO apt install -y unzip
  fi

  mapfile -t fonts < <(yaml_list fonts nerd)
  for font in "${fonts[@]}"; do
    [[ -z "$font" ]] && continue
    local name url
    name="${font%%|*}"
    url="$(echo "$font" | awk -F'url=' '{print $2}')"
    "$TARGET_BIN/omarchy-pop-fonts" --name "$name" --url "$url"
  done
}

symlink_with_backup() {
  local src="$1"
  local dst="$2"
  local backup_suffix="${3:-.omarchy-pop.bak}"

  mkdir -p "$(dirname "$dst")"

  if [[ -L "$dst" ]]; then
    local current_target
    current_target=$(readlink "$dst")
    if [[ "$current_target" == "$src" ]]; then
      echo "Already symlinked: $dst -> $src"
      return 0
    fi
    echo "Updating symlink: $dst -> $src"
    rm "$dst"
  elif [[ -e "$dst" ]]; then
    echo "Backing up existing $dst to $dst$backup_suffix"
    mv "$dst" "$dst$backup_suffix"
  fi

  ln -s "$src" "$dst"
  echo "Symlinked $dst -> $src"
}

sync_dotfiles() {
  local src_dir="$REPO_DIR/dotfiles"
  local config_dir="$TARGET_CONFIG"

  echo "Symlinking dotfiles to $config_dir..."
  
  # Ensure alias directory exists in ~/.config/shell/aliases/
  mkdir -p "$config_dir/shell/aliases"

  for item in "$src_dir"/*; do
    local name
    name=$(basename "$item")
    [[ "$name" == "home" ]] && continue

    # Symlink directory or file into ~/.config/
    # e.g. dotfiles/nvim -> ~/.config/nvim
    symlink_with_backup "$item" "$config_dir/$name"
  done
}

sync_home_dotfiles() {
  local home_src="$REPO_DIR/dotfiles/home"
  [[ -d "$home_src" ]] || return 0

  echo "Symlinking home dotfiles..."
  
  if [[ -f "$home_src/bashrc" ]]; then
    symlink_with_backup "$home_src/bashrc" "$HOME/.bashrc"
  fi
}

sync_themes() {
  mkdir -p "$TARGET_THEMES"
  rsync -av "$REPO_DIR/themes/" "$TARGET_THEMES/"
}

install_bin_scripts() {
  mkdir -p "$TARGET_BIN"
  install -m 755 "$REPO_DIR/bin/omarchy-pop-theme" "$TARGET_BIN/omarchy-pop-theme"
  install -m 755 "$REPO_DIR/bin/omarchy-pop-fonts" "$TARGET_BIN/omarchy-pop-fonts"
}

append_shell_snippet() {
  local terminal="$1"
  local rc_files=("$HOME/.bashrc" "$HOME/.zshrc")
  for rc in "${rc_files[@]}"; do
    [[ -f "$rc" ]] || continue
    if ! grep -q "omarchy-pop" "$rc" 2>/dev/null; then
      {
        printf '\n# omarchy-pop\n'
        printf 'export PATH="%s:$PATH"\n' "$TARGET_BIN"
        printf 'export TERMINAL=%s\n' "$terminal"
        printf '[ -f "%s" ] && source "%s"\n' "$TARGET_CONFIG/shell/omarchy-pop.sh" "$TARGET_CONFIG/shell/omarchy-pop.sh"
      } >> "$rc"
    fi
  done
}

apply_theme() {
  local theme="$1"
  if command -v omarchy-pop-theme >/dev/null 2>&1; then
    omarchy-pop-theme "$theme"
  else
    echo "omarchy-pop-theme not found on PATH; skipping theme apply"
  fi
}

run_fw_steps() {
  local fw_flag="$1" recovery_flag="$2"
  if [[ "$fw_flag" == "true" ]]; then
    echo "Running firmware updates (fwupdmgr)..."
    $SUDO fwupdmgr get-devices || true
    $SUDO fwupdmgr get-updates || true
    $SUDO fwupdmgr update || true
  fi
  if [[ "$recovery_flag" == "true" ]]; then
    echo "Updating Pop recovery partition..."
    $SUDO pop-upgrade recovery upgrade from-release || true
  fi
}

parse_meta() {
  declare -gA META
  META=()
  local meta="$1"
  IFS=',' read -ra parts <<< "$meta"
  for part in "${parts[@]}"; do
    [[ -z "$part" ]] && continue
    local key="${part%%=*}"
    local val="${part#*=}"
    META[$key]="$val"
  done
}

install_custom_list() {
  local section="$1" group="$2"
  mapfile -t items < <(yaml_installers "$section" "$group")
  for entry in "${items[@]}"; do
    [[ -z "$entry" ]] && continue
    IFS='|' read -r name meta <<< "$entry"
    parse_meta "$meta"
    local method="${META[method]:-manual}"
    local url="${META[url]:-}"
    local args="${META[args]:-}"
    local check="${META[check]:-}"
    local id="${META[id]:-}"
    local deb_url="${META[deb_url]:-${META[deb]:-}}"
    local note="${META[note]:-}"
    local npm_package="${META[npm_package]:-$name}"
    local bun_package="${META[bun_package]:-$name}"

    if [[ -n "$check" ]] && command -v "$check" >/dev/null 2>&1; then
      echo "$name already present (check: $check)"
      continue
    fi

    case "$method" in
      apt)
        install_apt_list "$name" "$name"
        ;;
      flatpak)
        if [[ -n "$id" ]]; then
          if ! flatpak list | awk '{print $1}' | grep -qx "$id"; then
            echo "Installing Flatpak $id for $name"
            flatpak install -y flathub "$id"
          else
            echo "Flatpak $id already installed"
          fi
        else
          echo "No Flatpak id for $name; skipping"
        fi
        ;;
      deb)
        if [[ -z "$deb_url" ]]; then
          echo "No deb_url for $name; skipping"
        else
          tmpdir=$(mktemp -d)
          trap 'rm -rf "$tmpdir"' EXIT
          echo "Installing $name from deb: $deb_url"
          (cd "$tmpdir" && curl -fsSLo pkg.deb "$deb_url")
          $SUDO dpkg -i "$tmpdir/pkg.deb" || $SUDO apt -f install -y
          rm -rf "$tmpdir"
          trap - EXIT
        fi
        ;;
      npm)
        if [[ -z "$npm_package" ]]; then
          echo "No npm package specified for $name; skipping"
        elif ! command -v npm >/dev/null 2>&1; then
          echo "npm not found; skipping npm install for $name"
        elif npm list -g "$npm_package" >/dev/null 2>&1; then
          echo "npm package $npm_package already installed"
        else
          echo "Installing npm package $npm_package for $name"
          npm install -g "$npm_package"
        fi
        ;;
      bun)
        if [[ -z "$bun_package" ]]; then
          echo "No bun package specified for $name; skipping"
        elif ! command -v bun >/dev/null 2>&1; then
          echo "bun not found; skipping bun install for $name"
        elif bun pm ls -g "$bun_package" >/dev/null 2>&1; then
          echo "bun package $bun_package already installed"
        else
          echo "Installing bun package $bun_package for $name"
          bun add -g "$bun_package" || bun install -g "$bun_package" || echo "bun install for $bun_package failed"
        fi
        ;;
      script)
        if [[ -z "$url" ]]; then
          echo "No script URL for $name; skipping"
        else
          echo "Running installer script for $name from $url"
          if [[ -n "$args" ]]; then
            curl -fsSL "$url" | bash -s -- $args
          else
            curl -fsSL "$url" | bash
          fi
        fi
        ;;
      custom_dangerzone)
        install_dangerzone
        ;;
      custom_antigravity)
        install_antigravity
        ;;
      manual|*)
        echo "Manual install for $name: ${note:-'no note provided'}"
        ;;
    esac
  done
}

install_uv_tools() {
  mapfile -t tools < <(yaml_top_list uv_tools)
  [[ ${#tools[@]} -eq 0 ]] && return
  if ! command -v uv >/dev/null 2>&1; then
    echo "uv not installed; skipping uv_tools"
    return
  fi
  for tool in "${tools[@]}"; do
    [[ -z "$tool" ]] && continue
    if uv tool list 2>/dev/null | grep -q "^$tool\b"; then
      echo "uv tool $tool already installed"
    else
      echo "Installing uv tool $tool"
      uv tool install "$tool" || echo "uv tool install $tool failed"
    fi
  done
}

install_power_management() {
  echo "Configuring power management (udev + powerprofilesctl)..."
  local helper_src="$REPO_DIR/bin/omarchy-power-helper"
  local helper_dst="/usr/local/bin/omarchy-power-helper"
  local rules_src="$REPO_DIR/omarchy/udev/99-omarchy-power.rules"
  local rules_dst="/etc/udev/rules.d/99-omarchy-power.rules"

  # Install helper
  if [[ -f "$helper_src" ]]; then
    if [[ ! -f "$helper_dst" ]] || ! cmp -s "$helper_src" "$helper_dst"; then
      echo "Installing $helper_dst..."
      $SUDO install -m 755 "$helper_src" "$helper_dst"
    fi
  else
    echo "Warning: $helper_src not found"
  fi

  # Install udev rules
  local rules_changed=false
  if [[ -f "$rules_src" ]]; then
    if [[ ! -f "$rules_dst" ]] || ! cmp -s "$rules_src" "$rules_dst"; then
      echo "Installing $rules_dst..."
      $SUDO install -m 644 "$rules_src" "$rules_dst"
      rules_changed=true
    fi
  else
    echo "Warning: $rules_src not found"
  fi

  if [[ "$rules_changed" == "true" ]]; then
    echo "Reloading udev rules..."
    $SUDO udevadm control --reload-rules
    $SUDO udevadm trigger
  fi

  # Apply current state once
  if [[ -x "$helper_dst" ]]; then
    echo "Applying initial power profile..."
    "$helper_dst" --apply || echo "Failed to apply power profile"
  fi
}

main() {
  need_file "$CONFIG_FILE"
  ensure_pyyaml

  local default_theme run_fw run_recovery ghostty_enabled
  default_theme="$(yaml_value defaults.theme osaka-jade)"
  run_fw="$(yaml_value defaults.run_fw_update false)"
  run_recovery="$(yaml_value defaults.run_recovery_upgrade false)"
  ghostty_enabled="$(yaml_value defaults.ghostty true)"

  echo "== omarchy-pop install =="

  run_fw_steps "$run_fw" "$run_recovery"

  mapfile -t apt_core < <(yaml_list apt core)
  mapfile -t apt_gui < <(yaml_list apt gui)
  mapfile -t apt_terminal < <(yaml_list apt terminal)

  install_apt_list "core" "${apt_core[@]}"
  install_apt_list "gui" "${apt_gui[@]}"

  local ghostty_done=false
  for entry in "${apt_terminal[@]}"; do
    [[ -z "$entry" ]] && continue
    IFS='|' read -r name meta <<< "$entry"
    if [[ "$name" == "ghostty" && "$ghostty_enabled" == "true" ]]; then
      install_ghostty && ghostty_done=true
    elif [[ "$name" != "ghostty" ]]; then
      install_apt_list "terminal" "$name"
    fi
  done

  if [[ "$ghostty_enabled" != "true" || "$ghostty_done" != true ]]; then
    if printf '%s\n' "${apt_terminal[@]}" | grep -q '^kitty'; then
      install_apt_list "terminal" "kitty"
    fi
  fi

  install_flatpaks
  install_bin_scripts
  install_fonts
  sync_dotfiles
  sync_home_dotfiles
  sync_themes

  install_custom_list installers runtimes
  install_custom_list installers ai_tools
  install_custom_list installers security
  install_uv_tools
  install_power_management

  local preferred_terminal="ghostty"
  if [[ "$ghostty_enabled" != "true" || "$ghostty_done" != true ]]; then
    preferred_terminal="kitty"
  fi
  append_shell_snippet "$preferred_terminal"
  apply_theme "$default_theme"

  echo "== Done. Log out/in for fonts and shell PATH to refresh. =="

  local hp_note="$(yaml_value hp_zbook_ultra.notes "")"
  local hp_emit="$(yaml_value hp_zbook_ultra.emit_notes false)"
  if [[ "$hp_emit" == "true" && -n "$hp_note" ]]; then
    echo "HP ZBook Ultra guidance: $hp_note"
  fi
}

main "$@"
