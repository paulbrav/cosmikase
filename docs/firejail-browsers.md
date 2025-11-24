# Running Brave or Firefox inside Firejail on Pop!_OS/Ubuntu

Here’s a sane, step‑by‑step way to run Brave or Firefox inside Firejail on Pop!_OS 24 (Ubuntu 24.04‑based).

---

## 1. Install Firejail + profiles

Open a terminal and run:

```bash
sudo apt update
sudo apt install firejail firejail-profiles
```

On Ubuntu 24.04 (and thus Pop!_OS 24), `firejail` provides the sandbox itself and `firejail-profiles` ships ready‑made profiles for lots of apps, including browsers. ([Ubuntu Updates][1])

Confirm it’s installed:

```bash
firejail --version
```

---

## 2. Make sure you’re using non‑Snap / non‑Flatpak browsers

Firejail works with “normal” executables (from `.deb`, AppImage, etc.), not with Flatpak/Snap sandboxes.

Quick checks:

```bash
which firefox
which brave-browser
snap list | grep -E 'firefox|brave'
flatpak list | grep -E 'firefox|brave'
```

If `which` shows something under `/usr/bin` and there’s no Snap/Flatpak entry, you’re good.

* **Firefox**: best is the official Mozilla `.deb` via the `packages.mozilla.org` APT repo (Mozilla’s own recommended method). ([Mozilla Support][2])
* **Brave**: use Brave’s `.deb` repo or the one‑command installer from brave.com/linux. ([Brave][3])

(Flatpak/Snap versions are *already* sandboxed and don’t mix cleanly with Firejail.)

---

## 3. Basic Firejail test with the browser

Firejail comes with profiles in `/etc/firejail` and automatically picks a profile based on the program name (e.g. `firefox.profile`, `brave-browser.profile`). ([Ubuntu Manpages][4])

### Firefox (DEB)

```bash
firejail firefox
```

### Brave

On most Debian/Ubuntu‑style installs, the executable is `brave-browser` (or `brave-browser-stable`):

```bash
firejail brave-browser
# or, if your binary is named differently:
# firejail brave-browser-stable
```

If the Brave profile is present, Firejail will read `/etc/firejail/brave-browser.profile`, which itself includes `brave.profile`. That profile whitelists Brave’s config dirs and (on modern packages) ensures Brave’s own sandbox can still see `/proc/config.gz`. ([Unix & Linux Stack Exchange][5])

### Check that the sandbox is actually running

In another terminal:

```bash
firejail --list
```

You should see a line containing `firefox` or `brave-browser` in a Firejail sandbox. ([Linux Hint][6])

---

## 4. Make the browser always start in Firejail (CLI + GUI)

### Option A – Command line only

You can simply always run:

```bash
firejail firefox
firejail brave-browser
```

### Option B – Use `firecfg` to integrate with the desktop

`firecfg` creates symlinks in `/usr/local/bin` and fixes `.desktop` files so apps with Firejail profiles run sandboxed when launched from menus, docks, etc. ([Man7][7])

Run once:

```bash
sudo firecfg
```

Then log out and back in.

Now, when you click the **Firefox** or **Brave** icon, Firejail should be used automatically (for any app that has a profile and is listed in `/etc/firejail/firecfg.config`).

To see what got linked:

```bash
firecfg --list
```

> **Note:** `firecfg` enables Firejail for *many* programs, not only browsers. If you’d rather only Firejail browsers, skip `firecfg` and do manual symlinks instead.

### Option C – Only sandbox browsers (manual symlinks)

If you want just Firefox + Brave sandboxed:

```bash
# Firefox
sudo ln -s /usr/bin/firejail /usr/local/bin/firefox

# Brave (adjust name if your binary differs)
sudo ln -s /usr/bin/firejail /usr/local/bin/brave-browser
```

Because `/usr/local/bin` is usually before `/usr/bin` in `$PATH`, running `firefox` or `brave-browser` (from terminal or desktop) now goes through Firejail, which then looks up the matching profile. ([Super User][8])

### Roll back `firecfg` or symlinks if something breaks

If `firecfg` causes issues:

```bash
sudo firecfg --clean
```

If you created manual symlinks, remove them:

```bash
sudo rm -f /usr/local/bin/firefox /usr/local/bin/brave-browser
```

---

## 5. “Private” browser homes (disposable or separate profiles)

Firejail’s `--private` features are perfect for browsers:

* `--private` → brand‑new, empty home dir in tmpfs, wiped when the browser closes. ([Arch Manual Pages][9])
* `--private=/some/dir` → treat that directory as `$HOME` for the browser; data persists there but is isolated from your real home. ([Arch Manual Pages][9])

### Disposable browser session

Good for opening random or untrusted sites:

```bash
firejail --private firefox
# or
firejail --private brave-browser
```

Every run is like a fresh profile; history, cookies, extensions, etc. vanish when you close it.

### Separate persistent “profile home”

Example: make an isolated “sandbox home” for Firefox:

```bash
mkdir -p ~/.sandboxes/firefox
firejail --private=$HOME/.sandboxes/firefox firefox
```

For Brave:

```bash
mkdir -p ~/.sandboxes/brave
firejail --private=$HOME/.sandboxes/brave brave-browser
```

From the browser’s point of view, `~/.sandboxes/<name>` *is* your home directory. All config, cache, and downloads go there and never touch your real `~`. ([Arch Manual Pages][9])

You can make launchers / aliases for these:

```bash
echo 'alias firefox-sbx="firejail --private=$HOME/.sandboxes/firefox firefox"' >> ~/.bashrc
echo 'alias brave-sbx="firejail --private=$HOME/.sandboxes/brave brave-browser"' >> ~/.bashrc
```

> If you use `--private=/path`, point the browser’s download directory to that path or whitelist your real `~/Downloads` so files don’t seem to “disappear”.

---

## 6. Verify or debug a profile

* **See running sandboxes:** `firejail --list`
* **Inspect the sandbox tree:** find the PID (`pgrep brave-browser`), then `firejail --tree <pid>` to see namespaces/mounts applied.
* **Audit a profile:** `firejail --audit brave-browser` (or `firefox`) shows what the profile would block/allow.
* **Bypass the profile for debugging:** `firejail --noprofile brave-browser` lets you compare behavior; if it fixes the issue, adjust your `.local` overrides.
* **Find the active profile:** `/etc/firejail/<app>.profile` plus any `~/.config/firejail/<app>.local` overrides.

---

## 7. Custom tweaks for Brave / Firefox

### Brave: keep its own sandbox active

Older Firejail setups used to break Brave’s internal sandbox by hiding `/proc/config.gz`, which made Brave think user namespaces weren’t available and forced `--no-sandbox`. ([GitHub][10])

Modern `brave.profile` files explicitly allow it:

```text
# Brave sandbox needs read access to /proc/config.gz
noblacklist /proc/config.gz
include chromium-common.profile
```

([Unix & Linux Stack Exchange][5])

If, for some reason, your profile doesn’t have that line, add it via a local override so updates don’t overwrite your changes:

```bash
mkdir -p ~/.config/firejail
nano ~/.config/firejail/brave.local
```

Add:

```text
noblacklist /proc/config.gz
```

Save and restart Brave via Firejail.

### Custom browser restrictions

You can tighten or loosen things by adding `.local` files:

```bash
mkdir -p ~/.config/firejail

# For Firefox
nano ~/.config/firejail/firefox.local

# For Brave
nano ~/.config/firejail/brave.local
```

Example snippets:

```text
# Block access to your real Documents:
blacklist ${HOME}/Documents

# Allow a specific downloads folder:
whitelist ${HOME}/Downloads
```

User‑level profiles in `~/.config/firejail` override the system ones in `/etc/firejail` and survive package updates. ([Ubuntu Manpages][4])

---

## 8. A few “gotchas”

* **Downloads location**: Profiles often only allow `~/Downloads`. If you use `--private=/path`, also point the browser downloads there or whitelist a folder you actually use. ([Firejail][11])
* **Screen sharing / portals**: Wayland screen sharing may fail; loosen the profile for PipeWire/portal sockets (see comments in `/etc/firejail/firefox.profile` or add targeted `whitelist` rules).
* **Hardware acceleration / DRM**: If video playback or Widevine breaks, try relaxing seccomp for the browser in a `.local` override or allow the relevant GPU/DRM device nodes.
* **Password managers / helpers**: Gnome keyring, 1Password, etc. may not be reachable. Whitelist their socket paths if needed.
* **AppArmor on Ubuntu/Pop!_OS**: Check `aa-status` to see if an AppArmor profile already wraps the browser; stacked confinement can block features. Adjust AppArmor or Firejail rules rather than disabling both.
* **Security expectations**: Firejail reduces what the browser can touch, but *running untrusted code is never fully safe*. Don’t treat it as magic armor. ([ArchWiki][12])

[1]: https://www.ubuntuupdates.org/package/core/noble/universe/base/firejail?utm_source=chatgpt.com "Package \"firejail\" (noble 24.04)"
[2]: https://support.mozilla.org/en-US/kb/install-firefox-linux?utm_source=chatgpt.com "Install Firefox on Linux - Mozilla Support"
[3]: https://brave.com/linux/?utm_source=chatgpt.com "Installing Brave on Linux"
[4]: https://manpages.ubuntu.com/manpages//bionic/man5/firejail-profile.5.html?utm_source=chatgpt.com "Security profile file syntax for Firejail"
[5]: https://unix.stackexchange.com/questions/671071/firejail-not-hiding-files-with-brave-browser?utm_source=chatgpt.com "Firejail not hiding files with Brave browser"
[6]: https://linuxhint.com/install_firejail_ubuntu/?utm_source=chatgpt.com "How to Install and Use Firejail in Ubuntu"
[7]: https://man7.org/linux/man-pages/man1/firecfg.1.html?utm_source=chatgpt.com "firecfg(1) - Linux manual page"
[8]: https://superuser.com/questions/1410654/how-do-i-create-a-single-symbolic-link-for-firejail-in-ubuntu-fedora-centos?utm_source=chatgpt.com "linux - How do I create a SINGLE symbolic link for Firejail ..."
[9]: https://man.archlinux.org/man/firejail.1?utm_source=chatgpt.com "firejail(1) - Arch manual pages"
[10]: https://github.com/netblue30/firejail/issues/2944?utm_source=chatgpt.com "Firejail breaks Brave browser default sandboxing #2944"
[11]: https://firejail.wordpress.com/documentation-2/firefox-guide/?utm_source=chatgpt.com "Firefox Sandboxing Guide - Firejail - WordPress.com"
[12]: https://wiki.archlinux.org/title/Firejail?utm_source=chatgpt.com "Firejail - ArchWiki"
