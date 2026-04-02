# COSMIC Multi-Mode Calculator — Design Spec

## Overview

A calculator application for the COSMIC desktop environment, built in Rust with libcosmic. Features three switchable modes (Standard, Engineering, Financial) with a Rust-native expression evaluation engine, full keyboard support, and unified calculation history.

Inspired by the [cosmic-utils/calculator](https://github.com/cosmic-utils/calculator) project for COSMIC integration patterns, but built clean-room with a multi-mode architecture from the start.

## Architecture

### Approach: Clean-Room with Reference

Fresh Rust/libcosmic project. The reference cosmic-utils/calculator is used only as a guide for COSMIC integration patterns (config, i18n, theming). No code is forked. No external evaluation dependency (no `qalc`).

### Project Structure

```
cosmic-ext-calculator/
├── Cargo.toml
├── src/
│   ├── main.rs                  # Entry point
│   ├── app.rs                   # CosmicApplication impl, top-level state
│   ├── app/
│   │   ├── config.rs            # Persistent config (history, preferences)
│   │   ├── keybinds.rs          # Keyboard shortcut definitions
│   │   └── localization.rs      # i18n via fluent
│   ├── engine/
│   │   ├── mod.rs               # Evaluate trait + shared types
│   │   ├── parser.rs            # Recursive descent expression parser
│   │   ├── standard.rs          # Arithmetic evaluation
│   │   ├── engineering.rs       # Trig, log, bitwise, base conversion
│   │   └── financial.rs         # TVM, amortization, compound interest
│   ├── ui/
│   │   ├── mod.rs               # Shared UI helpers, display formatting
│   │   ├── standard.rs          # Standard mode button grid
│   │   ├── engineering.rs       # Engineering mode layout
│   │   ├── financial.rs         # Financial mode layout
│   │   └── history.rs           # Unified history panel
│   └── core/
│       ├── mod.rs
│       ├── icons.rs             # Icon definitions
│       └── operators.rs         # Operator enum (shared across modes)
├── i18n/                        # Fluent translation files
└── res/                         # Icons, .desktop file
```

### Key Modules

- **`engine/`** — Evaluation logic. A trait `Evaluate` defines the interface; each mode has its own implementation. Standard and engineering share the recursive descent parser; financial has dedicated solvers.
- **`ui/`** — Each mode owns its button grid layout and maps button presses to engine calls. Shared display/input widget in `ui/mod.rs`.
- **`app.rs`** — Top-level state: active mode (`enum Mode { Standard, Engineering, Financial }`), current expression, display value, unified history.

## UI Layout

### Mode Switching

Tab bar at the top of the window with three tabs: **Standard**, **Engineering**, **Financial**. Switching tabs replaces the button grid below the shared display area. Keyboard: `Ctrl+1/2/3`.

### Shared Display Area

Present in all modes, above the button grid:
- Secondary line (smaller, dimmed): current expression being built
- Primary line (large, bold): current result or input value
- Right-aligned text, monospace font

### Standard Mode

Classic 4-column grid:

| C | ( | ) | % |
|---|---|---|---|
| 7 | 8 | 9 | ÷ |
| 4 | 5 | 6 | × |
| 1 | 2 | 3 | − |
| 0 | . | ⌫ | + |
| = (full width) ||||

### Engineering Mode

HP-48S-inspired layout: function menu rows (6 columns) stacked above the standard 4-column numpad.

**Function rows:**

| sin | cos | tan | sin⁻¹ | cos⁻¹ | tan⁻¹ |
|-----|-----|-----|-------|-------|-------|
| log | ln | x² | √x | xʸ | n! |

**Bitwise / Base row:**

| AND | OR | XOR | NOT | ≪ | ≫ |
|-----|-----|-----|-----|-----|-----|
| HEX | DEC | OCT | BIN | π | e |

**Below:** Standard 4-column numpad (C, parentheses, operators, digits, equals).

**Angle mode toggle:** DEG / RAD / GRAD segmented button, displayed between the display area and the function rows. Persisted in config.

### Financial Mode

Form-based layout for TVM, with quick function buttons and numpad.

**TVM fields** (label — input — Solve button per row):
- N (periods)
- I/Y % (interest rate)
- PV (present value)
- PMT (payment)
- FV (future value)

Fill in the known values, press "Solve" on the unknown. `Tab`/`Shift+Tab` cycles fields.

**Quick function buttons** (4 columns):

| Margin | Markup | Tax+ | Tax− |

**Below:** Numpad with C, ±, %, operators, digits, equals.

## Evaluation Engine

### Parser

Hand-rolled recursive descent parser. No external evaluation dependency.

**Operator precedence** (highest to lowest):
1. Parentheses / function calls
2. Unary operators (+, −)
3. Power (^, right-associative)
4. Multiply, divide, modulus
5. Add, subtract

**Implicit multiplication:** `2π` → `2*π`, `3(4+5)` → `3*(4+5)`.

### Evaluate Trait

```rust
pub trait Evaluate {
    fn evaluate(&self, expr: &str) -> Result<CalcResult, CalcError>;
}

pub struct CalcResult {
    pub value: f64,
    pub display: String,
    pub alt_bases: Option<AltBases>,
}

pub struct AltBases {
    pub hex: String,
    pub oct: String,
    pub bin: String,
}
```

### StandardEngine

Arithmetic: +, −, ×, ÷, modulus, parentheses, percentage.

### EngineeringEngine

Extends standard with:
- **Trigonometric:** sin, cos, tan, asin, acos, atan (respects angle mode)
- **Logarithmic:** log (base 10), ln (natural), log with custom base
- **Power/roots:** x², √x, xʸ, n!
- **Constants:** π, e
- **Bitwise:** AND, OR, XOR, NOT, left shift (≪), right shift (≫) — operate on integer part
- **Base conversion:** display result in HEX, DEC, OCT, BIN simultaneously via `AltBases`
- **Other:** abs, floor, ceil

### FinancialEngine

Standard arithmetic plus dedicated financial functions:

- **TVM solver:** Given any 4 of (N, I/Y, PV, PMT, FV), solve for the 5th. Rate solving uses Newton-Raphson iteration.
- **Percentage calculations:** markup, margin
- **Tax:** add tax (gross from net), remove tax (net from gross) — configurable tax rate
- **Compound interest:** direct calculation outside TVM context

### Error Handling

```rust
pub enum CalcError {
    DivisionByZero,
    InvalidExpression(String),
    DomainError(String),    // e.g., sqrt of negative, log of zero
    Overflow,
    ConvergenceError,       // TVM Newton-Raphson didn't converge
}
```

Errors display as COSMIC toast notifications.

## Keyboard Shortcuts

### Universal

| Key | Action |
|-----|--------|
| `0-9`, `.` | Number input |
| `+`, `-`, `*`, `/` | Operators |
| `Enter` / `=` | Evaluate |
| `Escape` | Clear |
| `Backspace` | Delete last character |
| `(`, `)` | Parentheses |
| `Ctrl+1` | Switch to Standard |
| `Ctrl+2` | Switch to Engineering |
| `Ctrl+3` | Switch to Financial |
| `Ctrl+H` | Toggle history panel |
| `Ctrl+Z` | Undo last input |
| `Ctrl+C` | Copy result to clipboard |
| `Ctrl+V` | Paste into expression |

### Engineering Mode

| Key | Action |
|-----|--------|
| Typed function names | `sin`, `cos`, `tan`, `log`, `ln`, etc. insert the function |

### Financial Mode

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Cycle through TVM input fields |

## History

- Unified `Vec<HistoryEntry>` across all modes
- Each entry: expression, result, mode (Standard/Engineering/Financial), timestamp
- Persisted via `cosmic_config`
- Displayed in a COSMIC context drawer (sidebar), toggled with `Ctrl+H` or toolbar button
- Clicking a history entry loads the result into the current expression
- Mode badge per entry: colored label ("STD", "ENG", "FIN")
- Clear all / delete individual entries supported

```rust
pub struct HistoryEntry {
    pub expression: String,
    pub result: String,
    pub mode: Mode,
    pub timestamp: i64,
}
```

## Dependencies

- **libcosmic** (pop-os/libcosmic) — COSMIC toolkit, theming, config, i18n infrastructure
- **serde** — serialization for config/history
- **i18n-embed-fl** + **rust-embed** — localization
- **tracing** — logging
- No external calculator binary (no qalc)

## COSMIC Integration

- Follows COSMIC app patterns: `cosmic::Application` trait, `cosmic_config` for persistence, Fluent for i18n
- Respects system theme (light/dark) automatically via libcosmic
- `.desktop` file for app launcher integration
- Standard COSMIC header bar with about dialog
