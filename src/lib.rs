//! Library surface for `cargo-cmake-bridge`.
//!
//! Exposes the same building blocks the binary uses, so other tools can
//! generate a `Find<Crate>.cmake` programmatically instead of shelling out.

pub mod cli;
pub mod generator;
pub mod metadata;
pub mod template;

pub use cli::CliArgs;
pub use metadata::{CrateInfo, LibKind};

/// Runs the full generate pipeline for the given parsed args, writing one
/// `Find<Crate>.cmake` per selected crate and returning the paths written.
pub fn run(args: &CliArgs) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let selected: Vec<Option<String>> = if args.all {
        metadata::list_crates_with_lib_target(&args.manifest_path)?
            .into_iter()
            .map(Some)
            .collect()
    } else if !args.crate_names.is_empty() {
        args.crate_names.iter().cloned().map(Some).collect()
    } else {
        vec![None]
    };

    let mut written = Vec::with_capacity(selected.len());
    for crate_name in selected {
        let crate_info = metadata::load(
            &args.manifest_path,
            args.target_dir.as_deref(),
            &args.profile,
            crate_name.as_deref(),
            args.generate_header,
        )?;
        let rendered = template::render(&crate_info);
        let crate_name_pascal = template::to_pascal_case(&crate_info.name);
        let out_path = generator::write_find_module(&args.out_dir, &crate_name_pascal, &rendered)?;
        written.push(out_path);
    }
    Ok(written)
}
