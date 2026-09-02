//! Entry point for `cargo cmake-bridge`.

use cargo_cmake_bridge::cli;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let args = cli::parse_args();
    for out_path in cargo_cmake_bridge::run(&args)? {
        println!("wrote {}", out_path.display());
    }
    Ok(())
}

