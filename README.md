# cargo-cmake-bridge

A Cargo subcommand that generates CMake `Find<Crate>.cmake` modules for Rust
crates, so a Rust static library can be linked into a legacy C++ CMake
project with a plain `find_package(<Crate>)`.

## Install

**Download a prebuilt binary** from the [Releases page](../../releases) —
grab the archive for your OS, extract `cargo-cmake-bridge`(`.exe`), and put
it on your `PATH` (Cargo picks up any `cargo-*` binary on `PATH` as a
subcommand automatically).

**Or build from source:**

```powershell
cargo install --path .
```

**Or use it as a library** in your own Rust tool instead of shelling out to
the binary:

```toml
[dependencies]
cargo-cmake-bridge = { git = "https://github.com/Cordagr/Cargo-Maker" }
```

```rust
use cargo_cmake_bridge::{cli::CliArgs, run};

let args = CliArgs::parse_from(["cargo-cmake-bridge", "--manifest-path", "Cargo.toml", "--out-dir", "cmake"]);
let written = run(&args)?;
```

## Usage

```powershell
# from the Rust crate's directory, after `cargo build --release`
cargo cmake-bridge --manifest-path Cargo.toml --profile release --out-dir ../cpp-project/cmake

# then in the C++ project's CMakeLists.txt
list(APPEND CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/cmake")
find_package(MyCrate REQUIRED)
target_link_libraries(my_app PRIVATE MyCrate::MyCrate)
```

Flags:

| Flag                | Required | Description                                                       |
|---------------------|----------|--------------------------------------------------------------------|
| `--manifest-path`   | yes      | Path to the crate's `Cargo.toml`                                    |
| `--out-dir`         | yes      | Directory to write `Find<Crate>.cmake` into                          |
| `--profile`         | no       | Build profile / target subdirectory (default `debug`)                |
| `--target-dir`      | no       | Override the cargo target directory                                  |
| `--crate`           | no       | Select a package by name; repeatable (`--crate a --crate b`)          |
| `--all`             | no       | Generate for every workspace member with a staticlib/cdylib target   |
| `--generate-header` | no       | Run `cbindgen` to (re)generate the C header before locating it       |

Both `--profile debug` and `--profile release` builds are probed automatically
regardless of which one you pass, so the generated `.cmake` also works with
multi-config generators (Visual Studio, Xcode) that switch configuration at
build time rather than at `cmake` configure time.

### Exposing headers and extra system libs

`cargo-cmake-bridge` looks for a C header at `include/<crate_name>.h` next to
the crate's `Cargo.toml` by default. To point at a different path, or to add
system libraries the CMake target should also link against, add this to the
crate's `Cargo.toml`:

```toml
[package.metadata.cmake-bridge]
header = "include/my_crate.h"
link_libs = ["m", "pthread"]
```

## Project structure

```
Cargo-Maker/
├── Cargo.toml                     # lib + bin manifest (bin name: cargo-cmake-bridge)
├── README.md                      # this file
├── CONTRIBUTING.md                # dev setup, tests, PR expectations
├── .gitignore
├── src/
│   ├── lib.rs                     # public library surface (cargo_cmake_bridge::run)
│   ├── main.rs                    # thin CLI wrapper around the library
│   ├── cli.rs                     # CLI argument parsing (clap)
│   ├── metadata.rs                # cargo metadata + build artifact/header discovery
│   ├── template.rs                # renders templates/FindCrate.cmake.tmpl
│   └── generator.rs               # writes the rendered file to disk
├── templates/
│   └── FindCrate.cmake.tmpl       # CMake find-module template (@VAR@ placeholders)
├── fixtures/
│   ├── sample-crate/              # tiny staticlib crate used by the integration test
│   └── multi-crate-workspace/     # two-crate workspace, used to test --all
├── examples/
│   └── cpp-consumer/              # minimal CMake project that links the sample crate
└── tests/
    └── generator_tests.rs         # end-to-end integration tests
```

## How a `cargo <subcommand>` binary works

Cargo discovers subcommands by looking for an executable named
`cargo-<subcommand>` on `PATH`. Running `cargo cmake-bridge <args>` is
equivalent to running `cargo-cmake-bridge <args>` directly, except Cargo
also passes the subcommand name itself as `argv[1]`, which `cli::parse_args`
strips before handing argv to clap.

## Non-goals (for v1)

- Multiple *different* libraries per generated file — each `Find<Crate>.cmake`
  still describes one crate; `--all` just runs the generator once per crate.
- Anything beyond a plain `cbindgen --output <path> <crate_dir>` invocation —
  project-specific `cbindgen` flags aren't exposed yet, use a `cbindgen.toml`
  in the crate for that (cbindgen picks it up automatically).
