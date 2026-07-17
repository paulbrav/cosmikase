# Troubleshooting Guide

Common issues and fixes for cosmikase. Everything routes through one orchestrator (chezmoi),
so most problems are an apply, a manifest edit, or a hardware gate.

## Table of Contents

- [Installation](#installation)
- [Package Installation](#package-installation)
- [Preflight & Hardware](#preflight--hardware)
- [Theme Switching](#theme-switching)
- [Chezmoi](#chezmoi)
- [Configuration](#configuration)
- [Recovery](#recovery)

---

## Installation

### chezmoi not found

`install.sh` installs the `chezmoi` binary to `~/.local/bin`. If it is not on your PATH
afterward:

```bash
export PATH="$HOME/.local/bin:$PATH"   # add to ~/.bashrc if missing
chezmoi --version
```

Re-running `./install.sh` is safe and idempotent.

### Prereqs missing

`install.sh` installs `curl`, `git`, and `python3-yaml` up front (single sudo prompt). If a
`run_onchange_` package script fails complaining a tool is missing, re-run the installer:

```bash
./install.sh
```

### Permission denied on a script

```bash
chmod +x bin/cosmikase-*
# or re-run the installer, which restores the managed copies
./install.sh
```

---

## Package Installation

### apt package fails

```bash
sudo apt update                 # refresh the package cache first
apt-cache search <package>      # confirm the name/availability
```

Fix the name in `cosmikase.yaml` if it is wrong, then re-run `./install.sh` (the package step
reruns when the manifest changes).

### Flatpak install fails

```bash
flatpak remote-list             # is flathub present?
flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
flatpak search <app-name>       # verify the application ID
```

The package script adds flatpak + the Flathub remote automatically when any flatpak app is
enabled; if that step was skipped (e.g. `COSMIKASE_CI=1`), add the remote by hand as above.

---

## Preflight & Hardware

`bin/cosmikase-preflight` prints a PASS/FAIL/WARN table. WARN is expected on non-COSMIC hosts
and before the webcam driver lands — it does not block the rest of the setup.

### Webcam WARN/FAIL

The HP ZBook Ultra G1a webcam (sensor OV05C10, AMD ISP4) needs the `amd_capture` driver, which
merged into **mainline Linux 7.2**. Pop!_OS 24.04 ships 6.17.9, so preflight will WARN until:

```bash
modinfo amd_capture                       # driver present?
ls /lib/firmware/amdgpu/isp_4_1_1.bin*     # firmware present?
uname -r                                   # kernel ≥ 7.2?
```

Interim paths: the Ubuntu **OEM kernel** (`linux-oem-24.04c`), a **DKMS backport** of
`amd_capture`, or a self-built **≥ 7.2** kernel. The sensor is an AMD ISP4 MIPI/CSI camera
(not USB UVC); it exposes a plain V4L2 node (`/dev/video0`) and needs no libcamera. Confirm
with `v4l2-ctl --list-devices` (from `v4l-utils`).

### Fingerprint FAIL

The HP ZBook Ultra G1a has a Synaptics fingerprint sensor. It works with `fprintd` after a
firmware update, once enabled in PAM:

```bash
fwupdmgr update                        # apply the sensor firmware update, then reboot
pam-auth-update --enable fprintd       # enable fingerprint auth in PAM
systemctl status fprintd
fprintd-enroll                         # enroll a finger once fprintd + the reader are present
```

### WiFi unstable

The MediaTek **MT7925** adapter's stability improved in **kernel 6.16+**; if you see drops on
an older kernel, moving to a newer kernel (the interim webcam kernels above all qualify) fixes
it.

### Power udev rule inactive

The power profile switcher (`cosmikase-power-helper`) is triggered by a udev rule installed
during apply. Verify and re-apply if needed:

```bash
cosmikase-power-helper          # run it manually to confirm it works
chezmoi apply                   # reinstall the managed udev rule
```

---

## Theme Switching

### Theme not found

```bash
ls ~/.local/share/cosmikase/themes/          # installed themes
ls ~/.local/share/cosmikase/themes/nord/     # verify one exists
```

Theme names are case-sensitive and must match a directory under `themes/`.

### Cursor theme not applying

1. Reload the window: `Ctrl+Shift+P` → "Developer: Reload Window".
2. Confirm the color-theme extension is installed:
   ```bash
   cursor --list-extensions | grep -i catppuccin
   cosmikase-cursor-extensions install
   ```
3. Check the setting: `grep colorTheme ~/.config/Cursor/User/settings.json`.

See [editor-theming.md](editor-theming.md).

### COSMIC theme not updating

```bash
systemctl --user restart cosmic-settings-daemon
cat ~/.config/cosmic/com.system76.CosmicTheme.Mode/v1/is_dark
```

Logging out and back in applies a theme fully if a component does not hot-reload. See
[cosmic-theming.md](cosmic-theming.md).

### Chezmoi conflict during a theme switch

```bash
chezmoi diff            # see what differs
chezmoi apply --force   # overwrite local edits with the managed version
```

Don't hand-edit chezmoi-managed files; edit the templates under `chezmoi/` instead.

### Rollback fails

Rollback reads `~/.config/cosmikase/theme-history`, created on the first switch. If it is
missing, switch back explicitly:

```bash
cosmikase-theme <previous-theme-name>
```

---

## Chezmoi

### Apply fails

```bash
chezmoi status     # what is out of date
chezmoi diff       # what would change
chezmoi doctor     # environment + template sanity
chezmoi apply --force
```

### Config (chezmoi.toml) looks wrong

`cosmikase-chezmoi` writes the theme into chezmoi's config atomically. Inspect or repair:

```bash
cat ~/.config/chezmoi/chezmoi.toml
cosmikase-chezmoi set theme nord      # rewrite the theme safely
```

Validate the TOML:

```bash
python3 -c "import tomllib,sys; tomllib.load(open(sys.argv[1],'rb'))" ~/.config/chezmoi/chezmoi.toml
```

If `cosmikase-chezmoi` reports that `uv` is missing, install it or re-run `./install.sh`.

---

## Configuration

### Invalid YAML in cosmikase.yaml

```bash
python3 -c "import yaml; yaml.safe_load(open('cosmikase.yaml'))"
```

Common causes: missing colons, tabs instead of spaces, or unquoted values containing colons.

### A package I enabled did not install

```bash
grep -n "<package>" cosmikase.yaml   # confirm it is present and install: true
./install.sh                         # the package step reruns on manifest change
sudo apt install <package>           # or install it directly to confirm availability
```

---

## Recovery

### Purge chezmoi-managed dotfiles

```bash
chezmoi diff     # preview what would be removed
chezmoi purge
```

Re-initialize with `chezmoi init --source ~/Repos/cosmikase/chezmoi --apply`.

### Uninstall cosmikase (keep packages)

```bash
chezmoi purge
rm -rf ~/.local/bin/cosmikase-* ~/.local/share/cosmikase ~/.config/chezmoi
```

Packages are not removed automatically. Remove any you no longer want with `apt`/`flatpak`.

---

## See Also

- [CLI Reference](cli-reference.md) — every command and its flags.
- [Configuration Reference](configuration-reference.md) — the `cosmikase.yaml` schema.
- [Editor Theming](editor-theming.md) · [COSMIC Theming](cosmic-theming.md)
