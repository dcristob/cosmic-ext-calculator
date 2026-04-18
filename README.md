# COSMIC Calculator

A multi-mode calculator for the [COSMIC desktop](https://system76.com/cosmic/), built with [libcosmic](https://github.com/pop-os/libcosmic).

Three calculators in one window:

- **Standard** — everyday arithmetic with parentheses and percentages.
- **Engineering** — HP-48S-inspired layout with trig, logs, powers, and bitwise operations.
- **Financial** — time-value-of-money solver (N, rate, PV, PMT, FV) plus quick tax add/remove helpers.

## Screenshots

_Coming soon._

## Install

### From source

Requires Rust (1.80+) and a COSMIC desktop session on Wayland.

```sh
git clone https://github.com/dcristob/cosmic-ext-calculator.git
cd cosmic-ext-calculator
cargo build --release
./target/release/cosmic-ext-calculator
```

### Desktop entry

To register the app with your desktop so it shows up in the launcher:

```sh
sudo install -Dm755 target/release/cosmic-ext-calculator /usr/bin/cosmic-ext-calculator
sudo install -Dm644 res/dev.dcristob.Calculator.desktop /usr/share/applications/dev.dcristob.Calculator.desktop
sudo install -Dm644 res/dev.dcristob.Calculator.metainfo.xml /usr/share/metainfo/dev.dcristob.Calculator.metainfo.xml
```

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
