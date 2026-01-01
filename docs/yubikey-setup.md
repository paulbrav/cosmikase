# YubiKey Setup for Pop!_OS and SSH

This guide covers two use cases:
1. **Local Pop!_OS 24:** YubiKey as a FIDO2 security key for PAM (sudo + polkit).
2. **Servers:** Using the same key as a second factor for SSH logins.

> Safety: have a recovery path before making the key required (a second/backup YubiKey, and a password-capable admin account or a TTY console you can reach if PAM gets misconfigured). Always test in a new terminal before logging out.

---

## 1. Pop!_OS 24: replicate the Cosmikase-style FIDO2 setup

The official Yubico stack for Linux uses:
* **libfido2** + **fido2-tools** to detect/manage the key. ([Launchpad][1])
* **pam-u2f (libpam-u2f + pamu2fcfg)** to integrate it as a second factor in PAM. ([Launchpad][2])

### 1.1 Install the packages

On Pop!_OS 24 (Ubuntu 24.04–based):

```bash
sudo apt update
sudo apt install libfido2-1 fido2-tools libpam-u2f pamu2fcfg yubikey-manager
```

* `libfido2-1` + `fido2-tools`: FIDO2 library + tools.
* `libpam-u2f` + `pamu2fcfg`: PAM module + helper to register your key.
* `yubikey-manager (ykman)`: convenient management of the YubiKey. ([Medium][3])

> **Security note:** ensure your system is up to date; there was a recent pam-u2f security fix. ([Linux Security][4])

### 1.2 Verify the YubiKey is seen

Plug the YubiKey into your machine and run:

```bash
fido2-token -L
```

You should see a line like:
`/dev/hidrawX: vendor=0x1050, product=0x0407 (Yubico YubiKey FIDO+CCID)`

### 1.3 Create the authfile (`/etc/fido2/fido2`)

Cosmikase uses `/etc/fido2/fido2` as the mapping file. To generate it manually:

```bash
sudo mkdir -p /etc/fido2
pamu2fcfg -u "$USER" | sudo tee /etc/fido2/fido2
```

You’ll be asked to **touch the YubiKey**. That writes a line with your user + key handle + public key into `/etc/fido2/fido2`. ([Launchpad][5])

To register multiple keys (backup keys), run again with `-a`:
```bash
pamu2fcfg -u "$USER" | sudo tee -a /etc/fido2/fido2
```

Lock it down:

```bash
sudo chmod 600 /etc/fido2/fido2
sudo chown root:root /etc/fido2/fido2
```

If you retire or lose a key, remove its line from `/etc/fido2/fido2` (or rebuild the file by re-running `pamu2fcfg` for the keys you still own).

### 1.4 Wire it into `sudo` (PAM)

Edit `/etc/pam.d/sudo` **carefully**:

```bash
sudoedit /etc/pam.d/sudo
```

Add this line *after* the primary auth line (`@include common-auth`):

```pam
# Require FIDO2 / U2F key as a second factor
auth required pam_u2f.so authfile=/etc/fido2/fido2 cue
```

Yubico recommends using `pam_u2f` as a **required** module so it acts as a second factor. ([Yubico Developers][6])

* `authfile=/etc/fido2/fido2` tells pam_u2f where your key mapping is.
* `cue` prompts you to “Please touch the device”.
* Keep a fallback: ensure you still have a password-capable admin user or a root shell you can reach (e.g., TTY) in case PAM needs to be reverted.

### 1.5 Wire it into polkit (`/etc/pam.d/polkit-1`)

Similarly:

```bash
sudoedit /etc/pam.d/polkit-1
```

Add:

```pam
@include common-auth
auth required pam_u2f.so authfile=/etc/fido2/fido2 cue
```

This ensures graphical elevation uses the key as well.

> Ordering matters: keep `@include common-auth` first, then `pam_u2f`. This guide intentionally does **not** change your GDM login screen; it only adds the key for sudo and polkit prompts.

### 1.6 Test before you log out

**Very important: don’t log out until you’ve tested.** Open a fresh terminal:

```bash
sudo -k          # drop cached sudo credentials
sudo echo "ok"
```

You should see:
1. Password prompt
2. “Touch your security key” cue
3. `ok` printed if you touch the key.

If anything breaks, revert by commenting out the `pam_u2f.so` lines.

### 1.7 Removal

To undo this:
```bash
sudoedit /etc/pam.d/sudo     # Remove/comment pam_u2f lines
sudoedit /etc/pam.d/polkit-1 # Remove/comment pam_u2f lines
sudo rm -rf /etc/fido2
sudo apt purge libpam-u2f pamu2fcfg fido2-tools
```

If you lose the key and cannot authenticate, drop to a root shell/TTY (single-user mode or recovery), comment out the `pam_u2f` lines in `sudo` and `polkit-1`, then log back in normally.

---

## 2. Using the YubiKey as 2FA for servers (SSH)

The best option in 2025 is **OpenSSH + FIDO2 “-sk” keys**. ([Yubico Developers][7])

### 2.1 Why FIDO2 SSH keys?

* **Keys never leave hardware:** Authentication uses the YubiKey each time.
* **User presence + PIN:** Requires touch and optional PIN.
* **No cloud dependency:** Purely SSH + key.
* **Supported:** Works on OpenSSH ≥ 8.2 (Ubuntu 20.04+).

### 2.2 Requirements

* **Client (Pop!_OS):** OpenSSH client, FIDO2 capable YubiKey.
* **Server:** OpenSSH server (Ubuntu 20.04+, Debian 11+, etc.).

### 2.3 Set YubiKey FIDO2 PIN (optional but recommended)

On Pop!_OS:
```bash
ykman fido access change-pin
```
(or `ykman fido access set-pin` if new). ([Medium][9])

### 2.4 Generate an SSH key on the YubiKey

On Pop!_OS:
```bash
ssh-keygen -t ed25519-sk -O resident -O verify-required -C "you@yourserver"
```

* `-t ed25519-sk`: Ed25519 key backed by security key.
* `-O resident`: Store key handle on device (portable).
* `-O verify-required`: Require PIN + touch.

You’ll be prompted to touch the key and enter PIN. This generates:
* `~/.ssh/id_ed25519_sk` (stub private key)
* `~/.ssh/id_ed25519_sk.pub` (public key)

### 2.5 Install key on server

```bash
ssh-copy-id -i ~/.ssh/id_ed25519_sk.pub user@your-server
```

### 2.6 Enforce 2FA semantics on server

To require **Password + Key** (True 2FA), edit `/etc/ssh/sshd_config` on the server:

```text
PubkeyAuthentication yes
PasswordAuthentication yes
AuthenticationMethods publickey,password
```

Then restart sshd: `sudo systemctl restart sshd`

If you prefer **Key Only (with PIN)**:
```text
PubkeyAuthentication yes
PasswordAuthentication no
```

### 2.7 Connect

From Pop!_OS:
```bash
ssh -i ~/.ssh/id_ed25519_sk user@your-server
```

You’ll be asked for PIN (if set) and touch.

### 2.8 Hardening + recovery tips

* Server SSH config: `PubkeyAuthentication yes`, `PasswordAuthentication no` (or `AuthenticationMethods publickey,password` if you truly need 2FA), `PermitRootLogin prohibit-password`, `MaxAuthTries 3`, `AllowAgentForwarding no`.
* Keep a backup admin path: an out-of-band console or a password-capable account that is *not* U2F-only, in case you lose the key.
* If the key is lost and SSH is locked down, use console access to temporarily enable `PasswordAuthentication yes`, log in, install a new key, then disable passwords again.

---

## 3. Troubleshooting quick hits

* **Device not seen:** `fido2-token -L` or `ykman info`. If nothing appears, check dmesg/`sudo udevadm monitor` and ensure `hidraw` access (on Ubuntu, `plugdev` rules normally cover this).
* **pam_u2f skipped:** Verify `/etc/fido2/fido2` exists, has a line for your user, and is `chmod 600` / owned by root.
* **SSH “provider not found”:** Set `SSH_SK_PROVIDER=/usr/lib/x86_64-linux-gnu/libfido2.so.1` (path can differ; check `dpkg -L libfido2-1`).
* **Multiple keys:** List handles with `pamu2fcfg -u "$USER" -L` and prune stale lines in `/etc/fido2/fido2`.

[1]: https://launchpad.net/ubuntu/%2Bsource/libfido2?utm_source=chatgpt.com "libfido2 package : Ubuntu - Launchpad"
[2]: https://launchpad.net/ubuntu/%2Bsource/pam-u2f?utm_source=chatgpt.com "pam-u2f package : Ubuntu - Launchpad"
[3]: https://kf106.medium.com/playing-around-with-a-yubikey-0450e048ae2b?utm_source=chatgpt.com "Playing around with a Yubikey - Keir Finlow-Bates - Medium"
[4]: https://linuxsecurity.com/advisories/ubuntu/ubuntu-7806-1-pam-u2f-px9e7jmchusd?utm_source=chatgpt.com "Ubuntu: PAM/U2F Critical Auth Bypass DoS USN-7806-1 ..."
[5]: https://launchpad.net/ubuntu/%2Bsource/pam-u2f/1.1.0-1.1%2Bdeb12u1build0.24.04.1?utm_source=chatgpt.com "1.1.0-1.1+deb12u1build0.24.04.1 : pam-u2f package : Ubuntu"
[6]: https://developers.yubico.com/pam-u2f/?utm_source=chatgpt.com "pam-u2f"
[7]: https://developers.yubico.com/SSH/Securing_SSH_with_FIDO2.html?utm_source=chatgpt.com "Securing SSH with FIDO2"
[9]: https://swjm.blog/the-complete-guide-to-ssh-with-fido2-security-keys-841063a04252?utm_source=chatgpt.com "The complete guide to SSH with FIDO2 security keys"
