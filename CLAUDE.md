# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

`wo.exe` is a small Windows console app (Rust) that prints where each person is, according to Home Assistant. UI text, error messages and the README are in German; keep new user-facing text German.

## Commands

```
cargo build --release        # -> target\release\wo.exe
cargo test --release         # only unit test: secret::tests::roundtrip (DPAPI)
cargo test roundtrip         # run a single test
.\build.ps1                  # build only
.\build.ps1 -Release -Version 0.2.1 -Notes "..."   # bump, build, commit, push, gh release
```

There is no lint config; use `cargo clippy` / `cargo fmt` if needed. The shell may block scripts: run with `powershell -ExecutionPolicy Bypass -File .\build.ps1`. In some sessions `gh` is installed but not on `PATH` (`C:\Program Files\GitHub CLI`).

## Architecture

- `src/main.rs`: argument handling, interactive `setup()`, and `run()` (parallel fetch via `thread::scope`, one thread per entity, order preserved). Arguments are normalized by stripping leading `/` and `-` and lowercasing, so `/reset`, `-reset`, `--reset`, `reset` are equivalent. The help text intentionally lists only the `/` forms.
- `src/ha.rs`: blocking `ureq` client. `list_trackers()` (`GET /api/states`, filtered to `person.*` / `device_tracker.*`) is used only during setup; normal runs call `GET /api/states/<entity_id>` per stored entity. Timeouts: 5 s connect, 10 s total. `State::location()` maps `home` -> "Zuhause", `not_home` -> "Unterwegs", other values pass through.
- `src/config.rs`: persists `%APPDATA%\wo\config.json` (`base_url`, `token_enc`, `entities`). The stored list is the entities the user *selected* to display (setup asks which to show, Enter = all), not an ignore list. Legacy 0.1.x configs with a plaintext `token` field are migrated (re-saved encrypted) on load.
- `src/secret.rs`: Windows DPAPI (`CryptProtectData`/`CryptUnprotectData` via `windows-sys`, hex-encoded) and hidden console input for the token. This makes the crate Windows-only.

## Release notes

`build.ps1 -Release` checks up front that the `v<Version>` release doesn't already exist, edits the first `version = "..."` in `Cargo.toml`, and commits/pushes only `Cargo.toml` and `Cargo.lock`. Commit any other changes (including `build.ps1` itself) before running it, since the tag is created from the pushed state.
