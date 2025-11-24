# Pop!_OS Backup Strategy

Here’s a solid, no-nonsense backup setup for Pop!_OS.

* **Scripts (CLI)** for your files
* **System snapshots** with a tool that plays nicely with Pop!_OS
* Optional: **“real” backups** with a modern tool (restic)

---

## 1. Understand what you’re backing up

On Pop!_OS you basically have three layers:

1. **User data** – your `/home/yourname` (documents, pictures, dotfiles, etc.).
2. **System config & package list** – `/etc`, and the list of installed packages.
3. **Whole system snapshots** – “roll back after an update broke things”.

System76’s own doc recommends: automatic, accessible, secure, and *distributed* backups (3 copies, one off‑site).([System76 Support][1])

We’ll use:

* **rsync script(s)** for your `/home` and configs
* **Timeshift** for *system snapshots*
* Optionally **restic** for encrypted/off‑site backups

---

## 2. Scripted backups of your home directory (rsync)

System76 already suggests `rsync` for command‑line backup.([System76 Support][1])

We’ll turn that into a reusable script.

### 2.1. Prepare a backup drive

1. Plug in an external disk or use another internal partition.
2. Mount it (via the GUI or `Disks`).

   After mounting, note the path – something like:

* `/media/youruser/BackupDrive`  *(GUI mount)*
* or `/mnt/backup`              *(manual/fstab)*

I’ll **use `/mnt/backup`** in the examples – **change this to your actual path**.

```bash
sudo mkdir -p /mnt/backup
# (only if you’re mounting it manually)
```

---

### 2.2. `backup-home-rsync.sh` script

Create the script:

```bash
nano ~/bin/backup-home-rsync.sh
```

Put this inside (adjust the DEST path):

```bash
#!/usr/bin/env bash
set -euo pipefail

# Where your backup drive is mounted:
DEST="/mnt/backup/home-backup"   # <-- change this!

# Make sure destination exists (and parent is mounted)
if [[ ! -d "$(dirname "$DEST")" ]]; then
  echo "Destination parent '$(dirname "$DEST")' does not exist or is not mounted." >&2
  exit 1
fi

# Refuse to run if the target mount is missing (prevents filling root disk)
if ! findmnt -rn "$(dirname "$DEST")" >/dev/null; then
  echo "Backup target not mounted at '$(dirname "$DEST")'." >&2
  exit 1
fi

mkdir -p "$DEST"

LOG_DIR="$HOME/.local/share/backup-logs"
mkdir -p "$LOG_DIR"
LOGFILE="$LOG_DIR/home-$(date +%F).log"

echo "Backing up $HOME to $DEST ..."
echo "Log: $LOGFILE"

# Core rsync options:
#  -a  archive (permissions, times, etc.)
#  -A  preserve ACLs
#  -X  preserve extended attributes
#  -H  preserve hard links
#  -v  verbose
RSYNC_OPTS="-aAXHv"

# Common excludes (tweak to taste)
EXCLUDES=(
  "--exclude=.cache/"
  "--exclude=Downloads/"
  "--exclude=.local/share/Trash/"
  "--exclude=*.iso"
)

# NOTE: This does NOT use --delete by default (safer).
# Once you are sure everything is correct, you can add --delete
# to RSYNC_OPTS if you want an exact mirror.

rsync $RSYNC_OPTS "${EXCLUDES[@]}" "$HOME/" "$DEST/" | tee "$LOGFILE"

echo "Done."
```

Make it executable:

```bash
chmod +x ~/bin/backup-home-rsync.sh
```

#### First run (safe mode)

Before trusting it, do a dry run:

```bash
rsync -aAXHvn \
  --exclude=.cache/ \
  --exclude=Downloads/ \
  --exclude=.local/share/Trash/ \
  "$HOME/" /mnt/backup/home-backup/
```

Check that it would copy what you expect.
If happy, run the script:

```bash
~/bin/backup-home-rsync.sh
```

### 2.3 Restore from the rsync backup

Do a dry-run first to see what would change:

```bash
rsync -aAXHvn /mnt/backup/home-backup/ "$HOME/"
```

If it looks right, run the real restore:

```bash
rsync -aAXHv /mnt/backup/home-backup/ "$HOME/"
```

---

## 3. Scripted backup of system configs & package list

You generally don’t need a full filesystem clone; restoring configs + reinstalling packages is enough. System76 explicitly suggests keeping a list of packages instead of backing up the binaries themselves.([System76 Support][1])

Create another script:

```bash
sudo nano /usr/local/sbin/backup-system-configs.sh
```

Contents:

```bash
#!/usr/bin/env bash
set -euo pipefail

DEST="/mnt/backup/system-config"   # <-- change if needed

if [[ $EUID -ne 0 ]]; then
  echo "Please run as root: sudo $0" >&2
  exit 1
fi

mkdir -p "$DEST/etc" "$DEST/var" "$DEST/pkg"

echo "Backing up /etc and some system data to $DEST ..."

# System-wide configs
rsync -aAXH /etc/ "$DEST/etc/"

# Package list (Debian/Ubuntu-style)
dpkg --get-selections > "$DEST/pkg/dpkg-selections.txt"

# Apt sources and states
rsync -aAXH /etc/apt/ "$DEST/pkg/apt/"
rsync -aAXH /var/lib/apt/ "$DEST/var/lib/apt/"
rsync -aAXH /var/lib/dpkg/ "$DEST/var/lib/dpkg/"

echo "Done. To reinstall packages later:"
echo "  sudo dpkg --set-selections < dpkg-selections.txt"
echo "  sudo apt-get dselect-upgrade"
```

Make it executable:

```bash
sudo chmod +x /usr/local/sbin/backup-system-configs.sh
```

Run it:

```bash
sudo backup-system-configs.sh
```

---

## 4. Automate the scripts (systemd timers or cron)

**Preferred:** systemd user timer (better logging + missed runs are caught up).

`~/.config/systemd/user/backup-home-rsync.service`

```ini
[Unit]
Description=Backup home via rsync (user)

[Service]
Type=oneshot
ExecStart=%h/bin/backup-home-rsync.sh
```

`~/.config/systemd/user/backup-home-rsync.timer`

```ini
[Unit]
Description=Daily home backup at 02:00

[Timer]
OnCalendar=*-*-* 02:00
Persistent=true

[Install]
WantedBy=timers.target
```

Enable:

```bash
systemctl --user daemon-reload
systemctl --user enable --now backup-home-rsync.timer
```

The mount guard in `backup-home-rsync.sh` prevents writing to `/` if the drive isn’t attached.

**If you prefer cron**, keep the mount guard in the script and add:

Open your user crontab:

```bash
crontab -e
```

Add, for example:

```cron
# Run home backup every day at 2:00
0 2 * * * /home/YOURUSER/bin/backup-home-rsync.sh

# Run system config backup every Sunday at 3:00 (needs sudo via sudoers if you want this)
0 3 * * 0 sudo /usr/local/sbin/backup-system-configs.sh
```

Replace `YOURUSER` and make sure the backup drive is mounted at those times.

---

## 5. System snapshots with Timeshift (for “oops, I broke the system”)

This is what you asked for as well: snapshots you can roll back to if system files get corrupted.

**Timeshift** is basically “System Restore” for Linux: it creates incremental snapshots of your system using rsync or Btrfs snapshots.([TeejeeTech][2])
There’s a Pop!_OS‑specific Timeshift guide showing how well it integrates.([FOSS Linux][3])

> **Important:** Timeshift is mainly for **system files** (root filesystem). Don’t rely on it as your only backup for personal files; use the rsync/restic stuff above for `/home`.([DEV Community][4])

### 5.1. Install Timeshift

```bash
sudo apt update
sudo apt install timeshift
```

([FOSS Linux][3])

Launch it from the app menu (“Timeshift”) or via:

```bash
sudo timeshift-gtk
```

### 5.2. Initial Timeshift setup

In the wizard:

1. **Snapshot type**

   * If your Pop!_OS installation uses **ext4** (default): pick **RSYNC**.
   * If you deliberately installed Pop!_OS on **Btrfs** with `@` and `@home` subvolumes, pick **BTRFS**.([mutschler.dev][5])

2. **Snapshot location**

   * Ideally a separate partition or external disk (not the same partition as `/` if you can avoid it).

3. **Schedule**

   * Enable daily/weekly snapshots (e.g., daily + keep last 5, weekly + last 3).

4. **User directories**

   * Typically **leave “Include user data” off**; that keeps Timeshift focused on system files and avoids huge snapshots.

### 5.3. Creating/Restoring snapshots (GUI)

* To **create** one manually: open Timeshift → “Create”.
* To **restore**:

  * Boot normally or from a Pop!_OS live USB.
  * Open Timeshift → pick a snapshot → “Restore” and follow the prompts.([FOSS Linux][3])

This will roll the *system* back; anything changed after that snapshot in `/` will be reverted.

> Timeshift works best on the system partition (ext4 or Btrfs root). Keep `/home` backups separate, and prune old snapshots in the Timeshift GUI or via `sudo timeshift --delete --tags D,W,M` if space runs low.

---

## 6. System snapshots from the command line (Timeshift CLI script)

If you want a **scriptable snapshot** (e.g., before big changes), Timeshift has a CLI:

Example manual command (RSYNC mode):

```bash
sudo timeshift --create \
  --comments "Before NVIDIA driver install" \
  --tags D
```

(`--tags D` = “daily” tag; you can also use `O` for on-demand, etc.)([LinuxQuestions][6])

You can wrap that in a small script:

```bash
sudo nano /usr/local/sbin/timeshift-on-demand.sh
```

```bash
#!/usr/bin/env bash
set -euo pipefail

COMMENT="${1:-On-demand snapshot}"

sudo timeshift --create \
  --comments "$COMMENT" \
  --tags O
```

```bash
sudo chmod +x /usr/local/sbin/timeshift-on-demand.sh
```

Usage:

```bash
sudo timeshift-on-demand.sh "Before kernel update"
```

---

## 7. Automatic snapshots before `apt` upgrades (optional but nice)

If you want **“take a snapshot every time I install/update packages”**, there’s a script called **`timeshift-autosnap-apt`**. It hooks into apt and runs Timeshift before any install/upgrade/remove.([GitHub][7])

On Pop!_OS (Ubuntu-based), typical install path is:

```bash
sudo apt install git make
git clone https://github.com/wmutschl/timeshift-autosnap-apt.git
cd timeshift-autosnap-apt
sudo make install
```

After that, when you run:

```bash
sudo apt upgrade
```

you’ll get an automatic Timeshift snapshot first (as long as Timeshift is configured and has a target device).

---

## 8. Optional: “real” backups with restic (encrypted, deduplicated, off‑site)

If you’d like something more robust than raw rsync (deduplication, encryption, cloud targets), **restic** is a great CLI backup tool.([restic.net][8])

### 8.1. Install restic

```bash
sudo apt update
sudo apt install restic
```

### 8.2. Initialize a repository on your backup disk

```bash
mkdir -p /mnt/backup/restic-repo      # adjust if needed
restic init -r /mnt/backup/restic-repo
```

You’ll be asked for a password – **do not lose this**.

### 8.3. Simple restic backup script

```bash
nano ~/bin/backup-home-restic.sh
```

```bash
#!/usr/bin/env bash
set -euo pipefail

export RESTIC_REPOSITORY="/mnt/backup/restic-repo"   # change path if needed
export RESTIC_PASSWORD_FILE="$HOME/.config/restic/pass"

# Ensure password file exists (chmod 600 that file beforehand)
if [[ ! -f "$RESTIC_PASSWORD_FILE" ]]; then
  echo "Missing $RESTIC_PASSWORD_FILE" >&2
  exit 1
fi

# What to back up
TARGETS=(
  "$HOME/Documents"
  "$HOME/Pictures"
  "$HOME/.config"
)

restic backup "${TARGETS[@]}"

# Optional: prune old snapshots
restic forget --keep-daily 7 --keep-weekly 4 --keep-monthly 6 --prune
```

Make executable:

```bash
chmod +x ~/bin/backup-home-restic.sh
```

Now you can add this to `cron` too, or alternate days with your rsync backup.

---

### 8.4 Restore from a restic backup

List snapshots, then restore to a temporary target:

```bash
restic snapshots
restic restore latest --target /tmp/restic-restore
```

Copy files you need back into place. Use `restic restore <id> --target …` to pick a specific snapshot. Always keep `RESTIC_PASSWORD_FILE` at `chmod 600` and store a sealed copy of that file (or the password) somewhere safe.

### 8.5 Encrypt/off-site the backup drive

* Encrypt external drives with LUKS (`sudo cryptsetup luksFormat /dev/sdX` then open and format) so stolen disks don’t leak data.
* Keep one copy off-site (another disk, cloud, or a restic repo in object storage). Don’t put the only restic password file on the same drive; keep a sealed backup of it separately.

---

## 9. Restore and integrity checks

* **Spot-check restores:** Regularly restore a single file from rsync (`rsync -aAXHvn SRC DEST`) and a folder from restic (`restic restore latest --target /tmp/test-restore`).
* **Integrity:** `rsync --checksum --itemize-changes --dry-run "$HOME/" /mnt/backup/home-backup/` to detect drift; `restic check` to verify repository health.
* **Mount check:** Confirm the backup disk is mounted before scheduled runs (script already exits if not).
* **Capacity:** Watch Timeshift/restic disk usage and prune old snapshots (`restic forget ... --prune`).

---

## 10. Very quick checklist

If you follow nothing else, **do at least this**:

1. ✅ Set up **Timeshift** with RSYNC snapshots for the system.
2. ✅ Use **`backup-home-rsync.sh`** (or Deja Dup GUI, which System76 also recommends([System76 Support][1])) to back up `/home`.
3. ✅ Store at least one copy **off‑site** (cloud or another physical location).
4. ✅ Occasionally **test a restore** (restore a single file from backup and boot one Timeshift snapshot) so you know it works.

[1]: https://support.system76.com/articles/backup-files/ "Back Up Files - System76 Support"
[2]: https://teejee2008.github.io/timeshift/?utm_source=chatgpt.com "Timeshift - GitHub Pages"
[3]: https://www.fosslinux.com/125641/how-to-backup-data-on-your-pop_os-using-timeshift.htm "How to Backup & Restore Data on Pop!_OS using TimeShift"
[4]: https://dev.to/dev-charodeyka/using-timeshift-for-systems-snapshots-and-recovery-on-debian-12-via-command-line-7m6?utm_source=chatgpt.com "Using Timeshift for System's Snapshots and Recovery on ..."
[5]: https://mutschler.dev/linux/pop-os-btrfs-22-04/ "Pop!_OS 22.04: installation guide with btrfs, luks encryption and auto snapshots with timeshift | mutschler.dev"
[6]: https://www.linuxquestions.org/questions/linux-software-2/timeshift-from-the-command-line-4175601482/?utm_source=chatgpt.com "TimeShift from the command line?"
[7]: https://github.com/wmutschl/timeshift-autosnap-apt?utm_source=chatgpt.com "GitHub - wmutschl/timeshift-autosnap-apt"
[8]: https://restic.net/?utm_source=chatgpt.com "restic · Backups done right!"
