//! Builds fixtures/sample-crate, runs the binary against it, and checks
//! the generated Find<Crate>.cmake for the expected paths and version.

use std::path::PathBuf;
use std::process::Command;

fn fixture_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("sample-crate")
        .join("Cargo.toml")
}

#[test]
fn generates_find_module_for_sample_crate() {
    let manifest_path = fixture_manifest();

    let build_status = Command::new(env!("CARGO"))
        .args(["build", "--manifest-path"])
        .arg(&manifest_path)
        .status()
        .expect("failed to run `cargo build` on fixture crate");
    assert!(build_status.success(), "fixture crate failed to build");

    let out_dir = std::env::temp_dir().join(format!("cargo-cmake-bridge-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out_dir);

    let status = Command::new(env!("CARGO_BIN_EXE_cargo-cmake-bridge"))
        .args(["cmake-bridge", "--manifest-path"])
        .arg(&manifest_path)
        .args(["--out-dir"])
        .arg(&out_dir)
        .args(["--profile", "debug"])
        .status()
        .expect("failed to run cargo-cmake-bridge");
    assert!(status.success(), "cargo-cmake-bridge exited with failure");

    let generated = out_dir.join("FindSampleCrate.cmake");
    let contents = std::fs::read_to_string(&generated)
        .unwrap_or_else(|e| panic!("expected {generated:?} to exist: {e}"));

    assert!(contents.contains("SAMPLE_CRATE_VERSION \"0.2.0\""));
    assert!(contents.contains("SampleCrate::SampleCrate"));
    assert!(contents.contains("sample_crate"));
    assert!(contents.contains("NAMES sample_crate.h"));

    let _ = std::fs::remove_dir_all(&out_dir);
}

fn workspace_manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("multi-crate-workspace")
        .join("Cargo.toml")
}

#[test]
fn generates_find_module_for_every_crate_with_all() {
    let manifest_path = workspace_manifest();

    let build_status = Command::new(env!("CARGO"))
        .args(["build", "--manifest-path"])
        .arg(&manifest_path)
        .status()
        .expect("failed to run `cargo build` on fixture workspace");
    assert!(build_status.success(), "fixture workspace failed to build");

    let out_dir = std::env::temp_dir().join(format!("cargo-cmake-bridge-test-all-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out_dir);

    let status = Command::new(env!("CARGO_BIN_EXE_cargo-cmake-bridge"))
        .args(["cmake-bridge", "--manifest-path"])
        .arg(&manifest_path)
        .args(["--out-dir"])
        .arg(&out_dir)
        .args(["--profile", "debug", "--all"])
        .status()
        .expect("failed to run cargo-cmake-bridge --all");
    assert!(status.success(), "cargo-cmake-bridge --all exited with failure");

    assert!(out_dir.join("FindCrateA.cmake").exists());
    assert!(out_dir.join("FindCrateB.cmake").exists());

    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn generate_header_reports_missing_cbindgen_clearly() {
    let manifest_path = fixture_manifest();

    // Build first: metadata::load() locates the library before it touches
    // headers, so an unbuilt fixture would fail for the wrong reason here.
    let build_status = Command::new(env!("CARGO"))
        .args(["build", "--manifest-path"])
        .arg(&manifest_path)
        .status()
        .expect("failed to run `cargo build` on fixture crate");
    assert!(build_status.success(), "fixture crate failed to build");

    let out_dir = std::env::temp_dir().join(format!("cargo-cmake-bridge-test-hdr-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out_dir);

    let output = Command::new(env!("CARGO_BIN_EXE_cargo-cmake-bridge"))
        .args(["cmake-bridge", "--manifest-path"])
        .arg(&manifest_path)
        .args(["--out-dir"])
        .arg(&out_dir)
        .args(["--generate-header"])
        .output()
        .expect("failed to run cargo-cmake-bridge --generate-header");

    // This assumes cbindgen isn't installed in the test environment. If it
    // is, this test just confirms the command didn't crash instead.
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("cbindgen"), "unexpected error: {stderr}");
    }

    let _ = std::fs::remove_dir_all(&out_dir);
}
