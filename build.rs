use std::path::PathBuf;

fn main() {
    let dst: PathBuf = cmake::Config::new("src/c/plwrapper")
        .define("CMAKE_BUILD_TYPE", "Release")
        .build();

    let lib_dir = dst.join("lib");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=plwrapper");
    println!("cargo:rustc-link-lib=dylib=pl");
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../lib");

    println!("cargo:rerun-if-changed=src/c/plwrapper/CMakeLists.txt");
    println!("cargo:rerun-if-changed=src/c/plwrapper/include/plwrapper.h");
    println!("cargo:rerun-if-changed=src/c/plwrapper/plwrapper.cc");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set."));
    let Some(profile_dir) = out_dir.ancestors().nth(3) else {
        return;
    };

    let source = lib_dir.join("libpl.so");
    let _ = std::fs::copy(&source, profile_dir.join("libpl.so"));
}
