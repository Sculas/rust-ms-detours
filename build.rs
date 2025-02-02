use cc::windows_registry::find_tool;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

// adopted and modified from https://github.com/compass-rs/sass-rs/blob/master/sass-sys/build.rs

macro_rules! t {
    ($e:expr) => {
        match $e {
            Ok(n) => n,
            Err(e) => panic!("\n{} failed with {}\n", stringify!($e), e),
        }
    };
}

fn main() {
    let src = get_detours_folder();
    let tool = find_tool("x86_64-pc-windows-msvc", "msbuild").expect("msbuild not found");

    let target = env::var("TARGET").expect("TARGET not found in environment");

    let mut msvc_platform = if target.contains("x86_64") {
        "x64"
    } else {
        "x86"
    };

    if target.starts_with("aarch64") {
        msvc_platform = "ARM64";
    }

    if target.starts_with("arm") {
        msvc_platform = "ARM";
    }

    let dest = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR not found in environment"));
    let build = dest.join("build");

    t!(fs::create_dir_all(&build));
    cp_r(&src, &build);

    fs::copy(
        env::current_dir().unwrap().join("wrapper.h"),
        build.join("wrapper.h"),
    )
    .unwrap();

    tool.to_command()
        .current_dir(&build)
        .args([
            "vc\\Detours.sln",
            "/p:Configuration=ReleaseMD",
            format!("/p:Platform={}", msvc_platform).as_str(),
        ])
        .status()
        .unwrap();

    // Tell cargo to look for shared libraries in the specified directory
    let target_folder = format!("lib.{}", msvc_platform);
    println!(
        "cargo:rustc-link-search={}",
        build.join(target_folder).display()
    );

    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
    println!("cargo:rustc-link-lib=detours");
    println!("cargo:rustc-link-lib=syelog");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");
}

fn cp_r(dir: &Path, dest: &Path) {
    for entry in t!(fs::read_dir(dir)) {
        let entry = t!(entry);
        let path = entry.path();
        let dst = dest.join(path.file_name().expect("Failed to get filename of path"));
        if t!(fs::metadata(&path)).is_file() {
            t!(fs::copy(path, dst));
        } else {
            t!(fs::create_dir_all(&dst));
            cp_r(&path, &dst);
        }
    }
}

fn get_detours_folder() -> PathBuf {
    env::current_dir()
        .expect("Failed to get the current directory")
        .join("detours")
}
