//! Build script for wvec
//!
//! Compiles Fortran code and links the shared library.

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

/// Returns the path to the Fortran source directory.
fn fortran_dir() -> PathBuf {
    // Get the project root dir;
    // CARGO_MANIFEST_DIR is a special variable that Cargo automatically provides when running build.rs.
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    manifest_dir.join("fortran")
}

/// Runs a command and panics if it fails.
fn run_or_panic(cmd: &mut Command, description: &str) {
    let status = cmd
        .status()
        .unwrap_or_else(|_| panic!("Failed to run: {}", description));
    if !status.success() {
        panic!("{} failed with status: {}", description, status);
    }
}

/// Adds a native library search path.
fn link_search_native(path: &Path) {
    println!("cargo:rustc-link-search=native={}", path.display());
}

/// Sets rpath so the binary can find shared libraries at runtime.
fn set_rpath(path: &Path) {
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path.display());
}

/// Links a dynamic library.
fn link_dylib(lib_name: &str) {
    println!("cargo:rustc-link-lib=dylib={}", lib_name);
}

/// Tells Cargo to rerun build script if path changes.
fn rerun_if_changed(path: &str) {
    println!("cargo:rerun-if-changed={}", path);
}

fn main() {
    let fortran_dir = fortran_dir();

    // Compile Fortran code
    run_or_panic(
        Command::new("make").current_dir(&fortran_dir),
        "Fortran compilation (make)",
    );

    // Link configuration
    link_search_native(&fortran_dir);
    set_rpath(&fortran_dir);
    link_dylib("wvec_core");
    link_dylib("openblas");
    link_dylib("gomp");

    // Rebuild triggers
    rerun_if_changed("fortran/");
}
