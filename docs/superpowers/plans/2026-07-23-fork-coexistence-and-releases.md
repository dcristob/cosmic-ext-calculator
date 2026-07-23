# Fork Coexistence + Versioned Releases Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rename this fork's binary to `cosmic-calc-plus` so it installs side-by-side with upstream `cosmic-ext-calculator`, rebrand the About screen, and add versioned GitHub releases alongside the existing rolling build.

**Architecture:** The collision is a single shared string — the crate/binary name `cosmic-ext-calculator`. Renaming the Cargo `package.name` rebinds the binary filename, the crate path (used in `tests/` and a `RUST_LOG` string), and the i18n-embed fluent domain (used to locate the `.ftl` file). All three must move together or the build/tests break or the app panics at launch. Release changes extend one workflow to fire on both `main` pushes (rolling) and `v*` tags (versioned).

**Tech Stack:** Rust 2024 edition, libcosmic, i18n-embed-fl (Fluent), GitHub Actions, `softprops/action-gh-release@v2`.

## Global Constraints

- Binary name: `cosmic-calc-plus` (verbatim, everywhere the old binary name appeared).
- Launcher/app display name: `Calc Plus`.
- App-id `dev.dcristob.Calculator` and all `res/dev.dcristob.Calculator.*` filenames: UNCHANGED.
- GitHub repo name / clone dir `cosmic-ext-calculator`: UNCHANGED (README URLs stay).
- Version: `0.3.0` in `Cargo.toml`; seed git tag `v0.3.0` after merge.
- After renaming the crate, NO occurrence of `cosmic_ext_calculator` (underscore) or the
  `cosmic-ext-calculator` binary name may remain in `src/`, `tests/`, `Cargo.toml`,
  `install.sh`, `res/*.desktop`, `res/*.metainfo.xml`, or `release.yml`.

---

### Task 1: Rename crate + i18n domain + crate-path references (build-green rename)

This is atomic: the Cargo name change simultaneously renames the binary, breaks the 4 test
files' `use cosmic_ext_calculator::…` imports, and changes the fluent domain (so the `.ftl`
must be renamed or `load_fallback_language(...).expect(...)` panics at startup). All land in
one commit so both `cargo build` and `cargo test` stay green.

**Files:**
- Modify: `Cargo.toml:2-3` (name, version)
- Rename: `i18n/en/cosmic_ext_calculator.ftl` → `i18n/en/cosmic_calc_plus.ftl`
- Modify: `tests/financial_engine_tests.rs:1`, `tests/parser_tests.rs:1-2`, `tests/engineering_engine_tests.rs:1-2`, `tests/standard_engine_tests.rs:1`
- Modify: `src/app/settings.rs:20`

**Interfaces:**
- Produces: crate path `cosmic_calc_plus::` (was `cosmic_ext_calculator::`); binary
  `target/release/cosmic-calc-plus`; fluent domain file `cosmic_calc_plus.ftl`.

- [ ] **Step 1: Baseline — confirm current tests pass before touching anything**

Run: `cargo test`
Expected: PASS (establishes the green baseline the rename must preserve).

- [ ] **Step 2: Edit `Cargo.toml` name and version**

Change lines 2-3 from:
```toml
name = "cosmic-ext-calculator"
version = "0.1.0"
```
to:
```toml
name = "cosmic-calc-plus"
version = "0.3.0"
```

- [ ] **Step 3: Rename the fluent file (preserve git history)**

Run:
```bash
git mv i18n/en/cosmic_ext_calculator.ftl i18n/en/cosmic_calc_plus.ftl
```
Expected: file renamed, staged.

- [ ] **Step 4: Update the 4 test files' crate-path imports**

In each file, replace the leading `cosmic_ext_calculator` with `cosmic_calc_plus`:
- `tests/financial_engine_tests.rs:1` → `use cosmic_calc_plus::engine::financial::{FinancialEngine, TvmParams, TvmSolveFor};`
- `tests/parser_tests.rs:1` → `use cosmic_calc_plus::engine::parser::Parser;`
- `tests/parser_tests.rs:2` → `use cosmic_calc_plus::engine::CalcError;`
- `tests/engineering_engine_tests.rs:1` → `use cosmic_calc_plus::engine::engineering::EngineeringEngine;`
- `tests/engineering_engine_tests.rs:2` → `use cosmic_calc_plus::engine::{AngleMode, Evaluate};`
- `tests/standard_engine_tests.rs:1` → `use cosmic_calc_plus::engine::{Evaluate, standard::StandardEngine};`

- [ ] **Step 5: Update the `RUST_LOG` target string**

`src/app/settings.rs:20` — change:
```rust
std::env::set_var("RUST_LOG", "cosmic_ext_calculator=info");
```
to:
```rust
std::env::set_var("RUST_LOG", "cosmic_calc_plus=info");
```

- [ ] **Step 6: Verify nothing references the old crate name**

Run: `grep -rn "cosmic_ext_calculator" src/ tests/ Cargo.toml i18n/`
Expected: no output (exit 1).

- [ ] **Step 7: Build and confirm the renamed binary exists**

Run: `cargo build --release && ls target/release/cosmic-calc-plus`
Expected: build succeeds; the file `target/release/cosmic-calc-plus` is listed.

- [ ] **Step 8: Run the full test suite**

Run: `cargo test`
Expected: PASS (same tests as the baseline, now under the new crate name).

- [ ] **Step 9: Commit**

```bash
git add Cargo.toml Cargo.lock i18n/ tests/ src/app/settings.rs
git commit -m "refactor: rename crate/binary to cosmic-calc-plus, bump to 0.3.0"
```

---

### Task 2: Rebrand the About screen and app title

**Files:**
- Modify: `src/app/mod.rs:277`
- Modify: `i18n/en/cosmic_calc_plus.ftl:1`

**Interfaces:**
- Consumes: `env!("CARGO_PKG_VERSION")` → `"0.3.0"` (from Task 1's `Cargo.toml`); fluent key
  `app-title` (feeds `.name(...)` at `mod.rs:275` and `set_window_title(...)` at `mod.rs:319`).

- [ ] **Step 1: Make the About version track Cargo**

`src/app/mod.rs:277` — change:
```rust
            .version("0.1.0")
```
to:
```rust
            .version(env!("CARGO_PKG_VERSION"))
```

- [ ] **Step 2: Rebrand the app title**

`i18n/en/cosmic_calc_plus.ftl:1` — change:
```
app-title = Calculator
```
to:
```
app-title = Calc Plus
```

- [ ] **Step 3: Build to confirm it compiles**

Run: `cargo build --release`
Expected: build succeeds.

- [ ] **Step 4: Verify the version macro resolves to 0.3.0**

Run: `cargo metadata --no-deps --format-version 1 | grep -o '"version":"0.3.0"' | head -1`
Expected: `"version":"0.3.0"` (confirms `env!("CARGO_PKG_VERSION")` will compile to 0.3.0).

- [ ] **Step 5: Commit**

```bash
git add src/app/mod.rs i18n/en/cosmic_calc_plus.ftl
git commit -m "feat: rebrand About screen and window title to Calc Plus 0.3.0"
```

---

### Task 3: Rename the binary in packaging resources and installer

**Files:**
- Modify: `res/dev.dcristob.Calculator.desktop` (`Exec`, `Name`)
- Modify: `res/dev.dcristob.Calculator.metainfo.xml:4` (`<name>`)
- Modify: `install.sh` (`BIN=`, header comment, echo text)

**Interfaces:**
- Consumes: binary name `cosmic-calc-plus` (Task 1). The `.desktop` `Exec` MUST equal the
  binary name so the launcher resolves to this fork.

- [ ] **Step 1: Update the desktop entry**

`res/dev.dcristob.Calculator.desktop` — change:
```
Name=Calculator
Exec=cosmic-ext-calculator
```
to:
```
Name=Calc Plus
Exec=cosmic-calc-plus
```

- [ ] **Step 2: Update the AppStream metainfo name**

`res/dev.dcristob.Calculator.metainfo.xml:4` — change:
```xml
  <name>Calculator</name>
```
to:
```xml
  <name>Calc Plus</name>
```

- [ ] **Step 3: Update the installer binary variable**

`install.sh` — change:
```bash
BIN="cosmic-ext-calculator"
```
to:
```bash
BIN="cosmic-calc-plus"
```

- [ ] **Step 4: Update the installer's header comment references**

`install.sh` lines 2-3 — change the two `cosmic-ext-calculator` mentions in the comment block
to `cosmic-calc-plus`:
```bash
# Install cosmic-calc-plus — from the latest GitHub release by default,
# or from a local build with --local.
```
(Leave the `REPO="dcristob/cosmic-ext-calculator"` line UNCHANGED — that is the repo name.)

- [ ] **Step 5: Verify the desktop Exec matches the binary and no stray binary refs remain**

Run:
```bash
grep -n "Exec=" res/dev.dcristob.Calculator.desktop
grep -rn "cosmic-ext-calculator" install.sh res/*.desktop res/*.metainfo.xml | grep -v "dcristob/cosmic-ext-calculator"
```
Expected: `Exec=cosmic-calc-plus`; second grep prints nothing (only repo-URL refs remain, which are filtered out).

- [ ] **Step 6: Syntax-check the installer**

Run: `bash -n install.sh`
Expected: no output (valid syntax).

- [ ] **Step 7: Commit**

```bash
git add install.sh res/dev.dcristob.Calculator.desktop res/dev.dcristob.Calculator.metainfo.xml
git commit -m "feat: point installer and launcher at cosmic-calc-plus binary"
```

---

### Task 4: Versioned + rolling releases in the workflow

**Files:**
- Modify: `.github/workflows/release.yml` (trigger, stage step, split publish into two)

**Interfaces:**
- Consumes: staged asset `dist/cosmic-calc-plus` (renamed from `cosmic-ext-calculator`).
- Produces: on `v*` tag → permanent release marked Latest; on `main` push → rolling
  `latest` prerelease. `install.sh` fetches `releases/latest/download/…` = the newest tag.

- [ ] **Step 1: Add the tag trigger**

`.github/workflows/release.yml` lines 3-6 — change:
```yaml
on:
  push:
    branches: [main]
  workflow_dispatch:
```
to:
```yaml
on:
  push:
    branches: [main]
    tags: ['v*']
  workflow_dispatch:
```

- [ ] **Step 2: Rename the staged binary**

`.github/workflows/release.yml:45` — change:
```yaml
          cp target/release/cosmic-ext-calculator dist/
```
to:
```yaml
          cp target/release/cosmic-calc-plus dist/
```

- [ ] **Step 3: Replace the single publish step with two conditional steps**

Replace the entire `Publish rolling "latest" release` step (lines 50-72) with:
```yaml
      - name: Publish rolling "latest" dev build
        if: github.ref == 'refs/heads/main'
        uses: softprops/action-gh-release@v2
        with:
          tag_name: latest
          name: "Latest dev build (main) — ${{ steps.meta.outputs.date }}"
          body: |
            Rolling build from the tip of `main`, replaced on every push.
            For a stable download, use a versioned release below.

            Built: ${{ steps.meta.outputs.date }}
            Commit: `${{ steps.meta.outputs.short_sha }}` (`${{ github.sha }}`)
          files: |
            dist/cosmic-calc-plus
            dist/dev.dcristob.Calculator.desktop
            dist/dev.dcristob.Calculator.metainfo.xml
            dist/dev.dcristob.Calculator.svg
          make_latest: "false"
          prerelease: true

      - name: Publish versioned release
        if: startsWith(github.ref, 'refs/tags/v')
        uses: softprops/action-gh-release@v2
        with:
          tag_name: ${{ github.ref_name }}
          name: "Calc Plus ${{ github.ref_name }}"
          generate_release_notes: true
          files: |
            dist/cosmic-calc-plus
            dist/dev.dcristob.Calculator.desktop
            dist/dev.dcristob.Calculator.metainfo.xml
            dist/dev.dcristob.Calculator.svg
          make_latest: "true"
          prerelease: false
```

- [ ] **Step 4: Validate the YAML parses**

Run: `python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/release.yml')); print('yaml ok')"`
Expected: `yaml ok`.

- [ ] **Step 5: Confirm no old binary name and both conditions present**

Run:
```bash
grep -n "cosmic-ext-calculator" .github/workflows/release.yml
grep -n "refs/heads/main\|refs/tags/v\|make_latest\|prerelease" .github/workflows/release.yml
```
Expected: first grep prints nothing; second shows both ref conditions and both `make_latest`/`prerelease` values.

- [ ] **Step 6: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add versioned releases on v* tags, keep rolling as prerelease"
```

---

### Task 5: Document the release convention (`CLAUDE.md`) and README note

**Files:**
- Create: `CLAUDE.md`
- Modify: `README.md:1` (title) and the "One-liner" section (note the installed binary name)

- [ ] **Step 1: Create `CLAUDE.md`**

```markdown
# Calc Plus (cosmic-calc-plus)

A fork of [cosmic-utils/calculator](https://github.com/cosmic-utils/calculator), rebranded so
it installs **side-by-side** with upstream (which owns the binary name `cosmic-ext-calculator`).

## Identity — do not collide with upstream
- Binary name: `cosmic-calc-plus` (upstream is `cosmic-ext-calculator`).
- App-id: `dev.dcristob.Calculator` (upstream is `dev.edfloreshz.Calculator`).
- Display name: `Calc Plus`.
- The crate name in `Cargo.toml` drives the binary name, the crate path used in `tests/`, and
  the i18n-embed fluent domain (the `i18n/en/<crate_name>.ftl` filename). Rename all together.

## Release convention
**Every merge to `main` must bump the version and cut a release.**
1. Bump `version` in `Cargo.toml`.
2. After merging, push a matching tag: `git tag v<version> && git push origin v<version>`.

Pushing to `main` only produces the rolling `latest` dev build (a prerelease). The `v*` tag is
what publishes the permanent, downloadable versioned release that the `install.sh` one-liner
serves. The About screen reads `env!("CARGO_PKG_VERSION")`, so bumping `Cargo.toml` keeps the
About screen, the release name, and the git tag in sync.
```

- [ ] **Step 2: Note the branding in the README**

`README.md:1` — change `# COSMIC Calculator` to `# Calc Plus`.
Then, in the "One-liner (prebuilt binary)" section, append one sentence after the first
paragraph:
```
Installs as `cosmic-calc-plus`, so it coexists with upstream `cosmic-ext-calculator`.
```

- [ ] **Step 3: Verify the files**

Run: `head -1 README.md && grep -c "release convention\|Every merge to" CLAUDE.md`
Expected: `# Calc Plus` and a non-zero count.

- [ ] **Step 4: Commit**

```bash
git add CLAUDE.md README.md
git commit -m "docs: add release convention and Calc Plus branding"
```

---

### Task 6: System cleanup, rebuild, and end-to-end verification (local, no commit)

This applies the fix on the user's machine and proves coexistence. No git commit.

**Files:** none (operates on `~/.local/bin` and the installed AUR package).

- [ ] **Step 1: Remove the stale shadow binary**

Run: `rm -v ~/.local/bin/cosmic-ext-calculator`
Expected: reports the file removed.

- [ ] **Step 2: Confirm upstream is now the only `cosmic-ext-calculator`**

Run: `which -a cosmic-ext-calculator`
Expected: only `/usr/bin/cosmic-ext-calculator` (the AUR package).

- [ ] **Step 3: Build and install the renamed fork locally**

Run:
```bash
cargo build --release
./install.sh --local
```
Expected: installs `cosmic-calc-plus` + desktop integration into `~/.local/`.

- [ ] **Step 4: Confirm both binaries coexist**

Run: `which cosmic-calc-plus; which -a cosmic-ext-calculator`
Expected: `~/.local/bin/cosmic-calc-plus` for the fork; only `/usr/bin/cosmic-ext-calculator` for upstream.

- [ ] **Step 5: Launch the fork and sanity-check branding**

Run: `cosmic-calc-plus &`
Expected: app launches without a fluent panic; window title reads `Calc Plus`; About shows `Calc Plus` / `0.3.0`. The COSMIC app menu shows two distinct entries: "Calculator" (upstream) and "Calc Plus" (fork).

---

## Post-merge release step (after this branch merges to `main`)

Not a code task — the first versioned release that seeds the `releases/latest` download URL:

```bash
git checkout main && git pull
git tag v0.3.0
git push origin v0.3.0
```

Then confirm the Actions run publishes a "Calc Plus v0.3.0" release marked **Latest** with all
four assets, and that `curl -fsSL …/install.sh | bash` installs it.
