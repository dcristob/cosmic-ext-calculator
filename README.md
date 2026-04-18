# COSMIC Calculator

A multi-mode calculator for the [COSMIC desktop](https://system76.com/cosmic/), built with [libcosmic](https://github.com/pop-os/libcosmic).

Three calculators in one window:

- **Standard** — everyday arithmetic with parentheses and percentages.
- **Engineering** — HP-48S-inspired layout with trig, logs, powers, and bitwise operations.
- **Financial** — time-value-of-money solver (N, rate, PV, PMT, FV) plus quick tax add/remove helpers.

## Screenshots

_Coming soon._

## Install

### One-liner (prebuilt binary)

Installs the latest release from GitHub into `~/.local/` (per-user) — no sudo needed:

```sh
curl -fsSL https://raw.githubusercontent.com/dcristob/cosmic-ext-calculator/main/install.sh | bash
```

System-wide (all users):

```sh
curl -fsSL https://raw.githubusercontent.com/dcristob/cosmic-ext-calculator/main/install.sh | sudo bash
```

The script fetches the binary, `.desktop` entry, AppStream metainfo, and icon from the [latest release](https://github.com/dcristob/cosmic-ext-calculator/releases/latest), and refreshes the icon/desktop caches so the app appears in your launcher immediately.

Binaries are built on Ubuntu (glibc-based). If your distro uses musl or an older glibc, build from source.

### From source

Requires Rust (1.80+) and a COSMIC desktop session on Wayland.

```sh
git clone https://github.com/dcristob/cosmic-ext-calculator.git
cd cosmic-ext-calculator
cargo build --release
./install.sh    # installs the freshly built binary + desktop integration
```

The `install.sh --local` flag installs from `target/release/` instead of fetching a release (use `sudo ./install.sh --local` for system-wide).

## Features

- Keyboard input for numbers, operators, parentheses, and `Enter` to evaluate.
- Mode switch shortcuts: **Ctrl+1** (Standard), **Ctrl+2** (Engineering), **Ctrl+3** (Financial).
- Undo (**Ctrl+Z**), copy result (**Ctrl+C**), toggle history panel (**Ctrl+H**).
- Persistent history across sessions.
- Angle mode selection (Deg/Rad/Grad) in engineering mode.
- Base conversion (Hex/Dec/Oct/Bin) and bitwise ops in engineering mode.
- TVM solver with per-field solve-for button in financial mode.

## Development

```sh
cargo check      # typecheck
cargo test       # unit tests
cargo run        # debug run
```

The code is organized as:

- `src/engine/` — pure calculation engines (Standard, Engineering, Financial) with no UI dependencies.
- `src/ui/` — button grids per mode.
- `src/app/` — application shell, message dispatch, config persistence.

## License

[GPL-3.0-only](LICENSE)
