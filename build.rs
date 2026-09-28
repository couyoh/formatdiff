use std::path::PathBuf;

fn main() {
    let dst: PathBuf = cmake::Config::new("src/c/plwrapper")
        .define("CMAKE_BUILD_TYPE", "Release")
        .build();

    let lib_dir = dst.join("lib");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=plwrapper");
    println!("cargo:rustc-link-lib=static=libwolv-io");
    println!("cargo:rustc-link-lib=static=wolv-utils");
    println!("cargo:rustc-link-lib=dylib=stdc++");

    println!("cargo:rerun-if-changed=src/c/plwrapper/CMakeLists.txt");
    println!("cargo:rerun-if-changed=src/c/plwrapper/include/plwrapper.h");
    println!("cargo:rerun-if-changed=src/c/plwrapper/plwrapper.cc");
}
