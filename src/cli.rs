//! CLI argument definitions for `cargo cmake-bridge`.

use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "cargo-cmake-bridge", bin_name = "cargo cmake-bridge")]
pub struct CliArgs {
    /// Path to the crate's Cargo.toml
    #[arg(long)]
    pub manifest_path: PathBuf,
    /// Cargo target directory (defaults to `cargo metadata`'s reported value)
    #[arg(long)]
    pub target_dir: Option<PathBuf>,
    /// Build profile to look up artifacts in
    #[arg(long, default_value = "debug")]
    pub profile: String,
    /// Directory to write the generated Find<Crate>.cmake into
    #[arg(long)]
    pub out_dir: PathBuf,
    /// Select a package by name; repeatable to generate several at once
    /// (defaults to the manifest's root package if omitted)
    #[arg(long = "crate", conflicts_with = "all")]
    pub crate_names: Vec<String>,
    /// Generate a Find<Crate>.cmake for every workspace member that has a
    /// staticlib or cdylib target
    #[arg(long)]
    pub all: bool,
    /// Run `cbindgen` to (re)generate the C header before locating it
    #[arg(long)]
    pub generate_header: bool,
}

pub fn parse_args() -> CliArgs {
    // Cargo invokes us as `cargo-cmake-bridge cmake-bridge <args>`, so drop
    // the injected subcommand token before handing argv to clap.
    let args = std::env::args()
        .enumerate()
        .filter(|(i, a)| !(*i == 1 && a == "cmake-bridge"))
        .map(|(_, a)| a);
    CliArgs::parse_from(args)
}
