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
