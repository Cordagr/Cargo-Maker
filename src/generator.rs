//! Writes the rendered CMake module to <out_dir>/Find<CrateName>.cmake.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn write_find_module(out_dir: &Path, crate_name_pascal: &str, contents: &str) -> Result<PathBuf> {
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("creating output directory {out_dir:?}"))?;

    let out_path = out_dir.join(format!("Find{crate_name_pascal}.cmake"));
    std::fs::write(&out_path, contents)
        .with_context(|| format!("writing {out_path:?}"))?;

    Ok(out_path)
}
