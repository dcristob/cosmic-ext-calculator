# Fork coexistence + versioned releases — design

**Date:** 2026-07-23
**Status:** Approved (pending spec review)

## Problem

Installing the AUR package `cosmic-ext-calculator-git` appears to install *this* fork
instead of upstream. Root cause (confirmed by inspection, not the PKGBUILD):

- This fork's binary is named `cosmic-ext-calculator` — identical to upstream's.
- A stale manual install from a `just`/`install.sh --local` run sits at
  `~/.local/bin/cosmic-ext-calculator` (not tracked by pacman).
- `~/.local/bin` precedes `/usr/bin` in `PATH`.
- Upstream's desktop entry launches via the **bare** command `Exec=cosmic-ext-calculator`,
  which resolves through `PATH` — so *both* upstream's and the fork's launchers run the
  fork's binary. The AUR package installed fine to `/usr/bin`; it is simply shadowed.

The fork is otherwise already distinct: app-id `dev.dcristob.Calculator`, and
`res/dev.dcristob.Calculator.{desktop,metainfo.xml,svg}` filenames do not collide with
upstream's `dev.edfloreshz.Calculator.*`.

## Goal

Make the fork a distinct application that installs **side-by-side** with upstream, and
publish proper downloadable releases so others can install it.

## Decisions

| Item | Value |
|------|-------|
| Binary name | `cosmic-calc-plus` (was `cosmic-ext-calculator`) |
| Launcher display name | `Calc Plus` (was `Calculator`) |
| App-id | `dev.dcristob.Calculator` (unchanged — already distinct) |
| GitHub repo name | `dcristob/cosmic-ext-calculator` (unchanged) |
| Version | `0.3.0` (bump `Cargo.toml`; seed git tag `v0.3.0`) |
| Release model | Versioned (git tags) **+** keep rolling `latest` |

## Part 1 — Binary rename (`cosmic-calc-plus`)

The single shared string `cosmic-ext-calculator` is the whole collision. Renaming the
Cargo `package.name` rebinds three derived identifiers: the binary filename, the crate
path (`cosmic_ext_calculator::` → `cosmic_calc_plus::`), and the i18n-embed fluent domain.
The first two are compiler-checked; the fluent domain is **not** — it fails at runtime.

**Critical:** `fluent_language_loader!()` (`src/core/localization.rs:14`) derives its domain
from `CARGO_PKG_NAME` and loads `i18n/en/<domain>.ftl`. `load_fallback_language(...).expect(...)`
**panics at startup** if the file is missing. The `.ftl` file must be renamed in lockstep.

Changes:
1. `Cargo.toml` — `name = "cosmic-calc-plus"`, `version = "0.3.0"`.
2. Rename `i18n/en/cosmic_ext_calculator.ftl` → `i18n/en/cosmic_calc_plus.ftl` (**required**;
   else startup panic).
3. `install.sh` — `BIN="cosmic-calc-plus"`; update header comment + echo text.
4. `res/dev.dcristob.Calculator.desktop` — `Exec=cosmic-calc-plus`, `Name=Calc Plus`.
5. `res/dev.dcristob.Calculator.metainfo.xml` — `<name>` → `Calc Plus`.
6. `.github/workflows/release.yml` — 2 references `cosmic-ext-calculator` → `cosmic-calc-plus`
   (`cp target/release/...` and the `files:` list).
7. `README.md` — binary-name mentions.

Verify no absolute `cosmic_ext_calculator::` crate-path references exist in tests/benches
(grep) — internal code uses `crate::`, so none expected.

## Part 2 — About screen

`src/app/mod.rs:274-288`:
- `.version("0.1.0")` → `.version(env!("CARGO_PKG_VERSION"))` — reads `Cargo.toml` at compile
  time; shows `0.3.0` and never drifts again.
- `app-title = Calculator` → `Calc Plus` in the renamed `.ftl`. This feeds both the About
  name and the window title (`set_window_title(fl!("app-title"))`, `mod.rs:319`).
- Repo/support links unchanged.

## Part 3 — Versioned + rolling releases

`.github/workflows/release.yml` already publishes a rolling `latest` release with four raw
assets on every push to `main` (this is what `install.sh` fetches from
`releases/latest/download/`). Add versioned releases without losing the rolling build.

```yaml
on:
  push:
    branches: [main]      # rolling dev build
    tags: ['v*']          # permanent versioned release
  workflow_dispatch:
```

Tag and branch pushes are mutually exclusive events; the build runs once, then one of two
publish steps fires by ref:

- **Tag push** (`refs/tags/v*`) → permanent release: `tag_name: ${{ github.ref_name }}`,
  `generate_release_notes: true`, `make_latest: "true"`, `prerelease: false`.
- **Main push** (`refs/heads/main`) → the existing rolling `latest` release, downgraded to
  `make_latest: "false"` + `prerelease: true` so it reads as a bleeding-edge dev build and
  does not fight versioned releases for the "Latest" badge.

Notes:
- `install.sh` fetches `releases/latest/download/…`, which GitHub resolves to the
  `make_latest` release — now the newest **version tag**, i.e. the latest *stable* build.
- That URL only resolves once the **first `v*` tag exists**. Seed it by pushing `v0.3.0`
  after merge.
- Asset filenames stay **unversioned** (`cosmic-calc-plus`, `dev.dcristob.Calculator.*`) on
  both release types, so `install.sh`'s fixed URLs keep working across versions.

## Part 4 — Release convention in `CLAUDE.md`

There is no project `CLAUDE.md` yet. Create one recording the convention:

> **Every merge to `main` must bump the version and cut a release.** Bump `version` in
> `Cargo.toml`, then push a matching `v<version>` git tag (e.g. `v0.3.0`) so the release
> workflow publishes a versioned release. `main` pushes only produce the rolling dev build;
> the tag is what creates the permanent, downloadable version.

Because the About screen reads `env!("CARGO_PKG_VERSION")`, bumping `Cargo.toml` is the
single action that keeps the About screen, the release name, and the git tag in sync.

## Part 5 — One-time system cleanup (outside the repo)

- `rm ~/.local/bin/cosmic-ext-calculator` — the stale shadow. After rebuild + reinstall,
  the fork lives at `~/.local/bin/cosmic-calc-plus`, and `/usr/bin/cosmic-ext-calculator`
  (AUR upstream) is reachable again.

## Verification

- `cargo build --release` produces `target/release/cosmic-calc-plus`; app launches without
  the fluent panic; About screen shows `Calc Plus` / `0.3.0`.
- `which -a cosmic-ext-calculator` → only `/usr/bin/...` (upstream).
- `which cosmic-calc-plus` → the fork.
- App menu shows two distinct entries: "Calculator" (upstream) and "Calc Plus" (fork).
- Pushing tag `v0.3.0` publishes a versioned release with notes + all four assets, marked
  Latest; `install.sh` one-liner installs it.

## Out of scope

- Renaming the GitHub repository.
- Cross-platform / multi-arch release builds (currently Linux x86_64 only).
- Publishing the fork to the AUR.
