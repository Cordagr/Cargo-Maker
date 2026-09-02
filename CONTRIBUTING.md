# Contributing

## Setup

Requires a stable Rust toolchain (edition 2021). No other tools are needed
to build and test the Rust side; CMake is only needed if you want to build
the example under `examples/cpp-consumer`.

```powershell
cargo build
cargo test
```

## Project layout

See the "Project structure" section in [README.md](README.md) for what each
file/module is responsible for.

## Running the integration test

`tests/generator_tests.rs` covers three things:

- `fixtures/sample-crate` (a tiny `staticlib` with a C header): builds it,
  runs `cargo-cmake-bridge` against it, and checks the generated
  `Find<Crate>.cmake`.
- `fixtures/multi-crate-workspace` (two `staticlib` crates in one workspace):
  runs `cargo-cmake-bridge --all` and checks both `.cmake` files got written.
- `--generate-header`: checks the error message is clear when `cbindgen`
  isn't installed (skips the assertion if it is installed).

```powershell
cargo test --test generator_tests
```

If you change the template in `templates/FindCrate.cmake.tmpl`, update the
assertions in that test to match.

## Trying the CMake consumer example

```powershell
cargo run -- --manifest-path fixtures/sample-crate/Cargo.toml --out-dir fixtures/sample-crate/cmake
cmake -S examples/cpp-consumer -B examples/cpp-consumer/build
cmake --build examples/cpp-consumer/build
```

## Making changes

- Keep `src/metadata.rs` free of anything CMake-specific — it should only
  ever produce a `CrateInfo`. CMake syntax belongs in `templates/` and
  `src/template.rs`.
- Prefer adding placeholders to `templates/FindCrate.cmake.tmpl` and wiring
  them in `template::render()` over hardcoding new logic into the template
  loader.
- Run `cargo test` before submitting a change; the integration test is the
  main safety net since there's no CMake test harness in CI yet.

## Reporting issues

Include the crate's `Cargo.toml`, the exact `cargo cmake-bridge` command you
ran, and the full error output.
