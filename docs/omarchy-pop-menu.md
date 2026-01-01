# omarchy-pop Menu (gum)

This repo provides a single interactive entrypoint, `omarchy-pop`, built with
[Charm gum](https://github.com/charmbracelet/gum), to make common actions discoverable.

## Requirements

- `gum` installed (`sudo apt install gum`)
- `omarchy-pop` scripts on your `PATH` (after `make install`, they are installed to `~/.local/bin`)

Verify:

```bash
command -v omarchy-pop
command -v gum
```

## Run

```bash
omarchy-pop
```

Options include:
- **Change Theme**: launches `theme-tui` if installed; otherwise prompts for a theme and runs `omarchy-pop-theme`.
- **Install Optional Software**: installs items marked `install: false` in `omarchy-pop.yaml`.
- **Setup Docker Databases**: starts PostgreSQL, MySQL, Redis, and/or MongoDB in Docker containers.
- **Update Everything**: runs `omarchy-pop-update`.

## Optional Software Installation

`omarchy-pop-install` reads **disabled** items from your config file and then installs the ones you select.

### Config file location

Run from the repo root (recommended), or pass the path explicitly:

```bash
omarchy-pop-install --config /path/to/omarchy-pop.yaml
```

Or set an environment variable:

```bash
export OMARCHY_POP_CONFIG=/path/to/omarchy-pop.yaml
omarchy-pop-install
```

### Undo

- **APT**:

```bash
sudo apt remove <package-name>
```

- **Flatpak**:

```bash
flatpak uninstall <app-id>
```

## Docker Databases

`omarchy-pop-databases` starts containers named:
- `omarchy-postgres`
- `omarchy-mysql`
- `omarchy-redis`
- `omarchy-mongodb`

It also creates Docker volumes for persistence, and will prompt you for passwords for databases that need them
(it provides a default value you can accept or replace).

### Verify

```bash
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
```

### Undo / cleanup

Stop + remove containers:

```bash
docker rm -f omarchy-postgres omarchy-mysql omarchy-redis omarchy-mongodb
```

Remove volumes (this deletes all DB data):

```bash
docker volume rm omarchy-postgres-data omarchy-mysql-data omarchy-redis-data omarchy-mongodb-data
```

## Troubleshooting

- **`gum` not found**: install it with `sudo apt install gum`.
- **`omarchy-pop` not found**: ensure `~/.local/bin` is on PATH and re-open your shell, or re-run `make install`.
- **`theme-tui` not found**: run `make setup` (dev) or `make install` (full), which installs the Python CLI tools via `uv`.


