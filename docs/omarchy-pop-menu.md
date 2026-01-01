# cosmikase Menu (gum)

This repo provides a single interactive entrypoint, `cosmikase`, built with
[Charm gum](https://github.com/charmbracelet/gum), to make common actions discoverable.

## Requirements

- `gum` installed (`sudo apt install gum`)
- `cosmikase` scripts on your `PATH` (after `make install`, they are installed to `~/.local/bin`)

Verify:

```bash
command -v cosmikase
command -v gum
```

## Run

```bash
cosmikase
```

Options include:
- **Change Theme**: launches `theme-tui` if installed; otherwise prompts for a theme and runs `cosmikase-theme`.
- **Install Optional Software**: installs items marked `install: false` in `cosmikase.yaml`.
- **Setup Docker Databases**: starts PostgreSQL, MySQL, Redis, and/or MongoDB in Docker containers.
- **Update Everything**: runs `cosmikase-update`.

## Optional Software Installation

`cosmikase-install` reads **disabled** items from your config file and then installs the ones you select.

### Config file location

Run from the repo root (recommended), or pass the path explicitly:

```bash
cosmikase-install --config /path/to/cosmikase.yaml
```

Or set an environment variable:

```bash
export COSMIKASE_CONFIG=/path/to/cosmikase.yaml
cosmikase-install
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

`cosmikase-databases` starts containers named:
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
- **`cosmikase` not found**: ensure `~/.local/bin` is on PATH and re-open your shell, or re-run `make install`.
- **`theme-tui` not found**: run `make setup` (dev) or `make install` (full), which installs the Python CLI tools via `uv`.


