//! Runs `cargo metadata` and locates the built static lib and its header.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LibKind {
    Static,
    Shared,
}

pub struct CrateInfo {
    pub name: String,
    pub version: String,
    pub lib_kind: LibKind,
    pub lib_path: PathBuf,
    /// Windows import library (`.dll.lib`) for a `cdylib`, if present.
    pub implib_path: Option<PathBuf>,
    /// Populated when a debug/release build is found alongside the requested
    /// profile, so multi-config generators (Visual Studio, Xcode) can pick
    /// the right artifact per configuration instead of just one profile.
    pub lib_path_debug: Option<PathBuf>,
    pub lib_path_release: Option<PathBuf>,
    pub header_paths: Vec<PathBuf>,
    pub link_libs: Vec<String>,
}

pub fn load(
    manifest_path: &Path,
    target_dir: Option<&Path>,
    profile: &str,
    crate_name_override: Option<&str>,
    generate_header: bool,
) -> Result<CrateInfo> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(manifest_path)
        .no_deps()
        .exec()
        .with_context(|| format!("running `cargo metadata` for {manifest_path:?}"))?;

    let package = match crate_name_override {
        Some(name) => metadata
            .packages
            .iter()
            .find(|p| p.name == name)
            .with_context(|| format!("crate `{name}` not found in workspace metadata"))?,
        None => metadata
            .root_package()
            .context("no root package found; pass --crate to select one explicitly")?,
    };

    let target_dir: PathBuf = target_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| metadata.target_directory.clone().into_std_path_buf());

    let lib_kind = detect_lib_kind(package)?;
    // cargo normalizes crate names with '-' to '_' in output artifact filenames.
    let lib_stem = package.name.replace('-', "_");
    let profile_dir = target_dir.join(profile);
    let lib_path = find_lib(&profile_dir, lib_kind, &lib_stem).with_context(|| {
        format!(
            "no {} library found for `{lib_stem}` in {profile_dir:?}; did you run `cargo build --profile {profile}`?",
            match lib_kind {
                LibKind::Static => "static",
                LibKind::Shared => "shared",
            }
        )
    })?;

    let implib_path = match lib_kind {
        LibKind::Shared => find_implib(&profile_dir, &lib_stem),
        LibKind::Static => None,
    };
    let lib_path_debug = find_lib(&target_dir.join("debug"), lib_kind, &lib_stem);
    let lib_path_release = find_lib(&target_dir.join("release"), lib_kind, &lib_stem);

    let header_paths = if generate_header {
        vec![generate_header_with_cbindgen(package)?]
    } else {
        discover_header(package).into_iter().collect()
    };
    let link_libs = discover_link_libs(package);

    Ok(CrateInfo {
        name: package.name.clone(),
        version: package.version.to_string(),
        lib_kind,
        lib_path,
        implib_path,
        lib_path_debug,
        lib_path_release,
        header_paths,
        link_libs,
    })
}

/// Names of workspace members that have a `staticlib` or `cdylib` target,
/// for `--all`.
pub fn list_crates_with_lib_target(manifest_path: &Path) -> Result<Vec<String>> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(manifest_path)
        .no_deps()
        .exec()
        .with_context(|| format!("running `cargo metadata` for {manifest_path:?}"))?;

    Ok(metadata
        .packages
        .iter()
        .filter(|p| detect_lib_kind(p).is_ok())
        .map(|p| p.name.clone())
        .collect())
}

fn detect_lib_kind(package: &cargo_metadata::Package) -> Result<LibKind> {
    let kinds: Vec<&str> = package
        .targets
        .iter()
        .flat_map(|t| t.kind.iter())
        .map(String::as_str)
        .collect();

    if kinds.contains(&"staticlib") {
        Ok(LibKind::Static)
    } else if kinds.contains(&"cdylib") {
        Ok(LibKind::Shared)
    } else {
        bail!(
            "crate `{}` has no [lib] crate-type of \"staticlib\" or \"cdylib\"",
            package.name
        )
    }
}

/// Looks for a C header two ways: an explicit path in
/// `[package.metadata.cmake-bridge] header = "..."`, or the conventional
/// `include/<crate_name>.h` next to Cargo.toml (what `cbindgen` typically
/// produces). The manifest path wins if both are present.
fn discover_header(package: &cargo_metadata::Package) -> Option<PathBuf> {
    let crate_root = package.manifest_path.parent()?.as_std_path();

    let configured = package
        .metadata
        .get("cmake-bridge")
        .and_then(|v| v.get("header"))
        .and_then(|v| v.as_str())
        .map(|rel| crate_root.join(rel));

    let conventional = crate_root
        .join("include")
        .join(format!("{}.h", package.name.replace('-', "_")));

    configured.into_iter().chain(std::iter::once(conventional)).find(|p| p.exists())
}

/// Where a header should live: the configured path if set, otherwise the
/// `include/<crate_name>.h` convention. Doesn't check whether it exists yet
/// -- used by both discovery and cbindgen generation.
fn header_target_path(package: &cargo_metadata::Package) -> Result<PathBuf> {
    let crate_root = package
        .manifest_path
        .parent()
        .context("crate manifest has no parent directory")?
        .as_std_path();

    let configured = package
        .metadata
        .get("cmake-bridge")
        .and_then(|v| v.get("header"))
        .and_then(|v| v.as_str())
        .map(|rel| crate_root.join(rel));

    Ok(configured.unwrap_or_else(|| {
        crate_root
            .join("include")
            .join(format!("{}.h", package.name.replace('-', "_")))
    }))
}

fn generate_header_with_cbindgen(package: &cargo_metadata::Package) -> Result<PathBuf> {
    let crate_root = package
        .manifest_path
        .parent()
        .context("crate manifest has no parent directory")?
        .as_std_path();
    let out_path = header_target_path(package)?;
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating header output directory {parent:?}"))?;
    }

    let status = std::process::Command::new("cbindgen")
        .arg("--output")
        .arg(&out_path)
        .arg(crate_root)
        .status();

    match status {
        Ok(status) if status.success() => Ok(out_path),
        Ok(status) => bail!("cbindgen exited with {status} while generating {out_path:?}"),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            bail!("cbindgen not found on PATH; install it with `cargo install cbindgen`")
        }
        Err(err) => Err(err).with_context(|| format!("running cbindgen for {}", package.name)),
    }
}

fn discover_link_libs(package: &cargo_metadata::Package) -> Vec<String> {
    package
        .metadata
        .get("cmake-bridge")
        .and_then(|v| v.get("link_libs"))
        .and_then(|v| v.as_array())
        .map(|libs| libs.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

fn find_lib(dir: &Path, kind: LibKind, lib_stem: &str) -> Option<PathBuf> {
    let candidates: &[String] = &match kind {
        LibKind::Static => [format!("lib{lib_stem}.a"), format!("{lib_stem}.lib")].to_vec(),
        LibKind::Shared => [
            format!("lib{lib_stem}.so"),
            format!("lib{lib_stem}.dylib"),
            format!("{lib_stem}.dll"),
        ]
        .to_vec(),
    };

    candidates.iter().map(|name| dir.join(name)).find(|p| p.exists())
}

/// Windows pairs a `.dll` with a separate `.dll.lib` (or `<name>.lib`)
/// import library that the linker needs; Unix shared libs don't have one.
fn find_implib(dir: &Path, lib_stem: &str) -> Option<PathBuf> {
    [
        dir.join(format!("{lib_stem}.dll.lib")),
        dir.join(format!("{lib_stem}.lib")),
    ]
    .into_iter()
    .find(|p| p.exists())
}

