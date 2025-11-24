# Repository Guidelines

Use these conventions to keep contributions consistent and easy to review.

## Project Structure & Module Organization
- Place runtime code in `src/`; keep entrypoints or CLIs in `cmd/` or `app/` and shared helpers in `scripts/`.
- Mirror modules with tests in `tests/` (e.g., `src/example.py` → `tests/test_example.py`, or `src/example.rs` → `tests/example_tests.rs`).
- Store docs in `docs/` and assets in `assets/`. Keep configuration in versioned files (e.g., `.editorconfig`, `pyproject.toml`, `package.json`, `Cargo.toml`) and provide `.env.example` for environment keys.

## Build, Test, and Development Commands
- Prefer Make targets or wrapper scripts so everyone runs the same steps. Suggested defaults: `make setup` (install deps), `make dev` (run locally), `make test` (full suite), `make fmt` and `make lint` (format + static checks).
- If Make targets are absent, use the stack’s native commands (e.g., `pip install -r requirements.txt` / `npm install`, `pytest` / `npm test`), and add the wrappers to a `Makefile` for repeatability.

## Coding Style & Naming Conventions
- Default to 4-space indentation and a 100-character line guide unless a formatter enforces otherwise.
- Use `snake_case` for files, functions, and variables; `PascalCase` for classes/types; `UPPER_SNAKE_CASE` for constants and env vars.
- Run formatters and linters before committing (e.g., `ruff`/`black`, `eslint`/`prettier`, or `cargo fmt`/`cargo clippy` based on language). Do not introduce lint warnings.

## Testing Guidelines
- Keep tests close to the code they cover under `tests/` and name them after the module under test.
- Prefer fast, deterministic tests; avoid network calls or system mutations. Use fakes/mocks for external services.
- Add regression tests when fixing bugs. Target high coverage on critical paths; required checks must pass.

## Documentation Guidelines
- Pair every “do X” with “undo/restore/verify X” (e.g., backups need restore steps; config changes need rollback notes).
- Call out safety rails for operational steps (mount checks before writing, fallbacks/recovery users for auth changes).
- Include quick validation or troubleshooting snippets so readers can confirm behavior and debug common breakages.
- Note platform-specific limits or conflicts (e.g., Timeshift scope, AppArmor/Firejail stacking).

## Commit & Pull Request Guidelines
- Write commits in imperative mood (`Add CI pipeline`, `Fix input validation`). Keep summaries under ~72 characters and include a short body for rationale or edge cases.
- Open PRs with a clear description, linked issue numbers, and any screenshots or logs that help reviewers. List what changed, how it was tested, and any follow-up work.
- Keep PRs focused and small; avoid unrelated refactors. Address review feedback promptly and document decisions.

## Security & Configuration Tips
- Never commit secrets or personal tokens; load them via environment variables and keep a sanitized `.env.example` updated.
- Pin dependencies where practical and review new packages for licensing and security risk before adding them.
