use std::path::Path;
use regex::Regex;

fn main() {
    // Generate CXX bridge
    let mut bridge = cxx_build::bridge("src/lib.rs");

    bridge
        .file("src/aria2_bridge.cpp")
        .flag_if_supported("-std=c++14");

    if cfg!(debug_assertions) {
        bridge.flag_if_supported("-O3");
    }

    let libaria2_dir = option_env!("LIBARIA2_DIR");

    match libaria2_dir {
        None => {
            // Link with aria2 lib installed as a package
            pkg_config::Config::new()
                .atleast_version("1.35.0")
                .probe("libaria2")
                .expect("Dependency not satisfied: aria2 can't be found! Run:  sudo apt install libaria2-0-dev");
            println!("cargo:rustc-link-lib=aria2");
        }
        Some(libaria2_dir) => {
            // Link with aria lib from a local path.
            // Must contain <path>/include/aria2/aria2.h and <path>/lib/*.a files
            let libaria2_dir = Path::new(libaria2_dir);
            let lib_dir = libaria2_dir.join("lib");
            let include_dir = libaria2_dir.join("include");
            if !libaria2_dir.exists() || !lib_dir.exists() || !include_dir.exists() {
                panic!("Path in LIBARIA2_DIR env var does not exist, or does not contain 'lib' and 'include' sub-dirs")
            }
            bridge.include(include_dir);
            // Must use canonical path for linking
            println!("cargo:rustc-link-search=native={}", lib_dir.canonicalize().unwrap().to_str().unwrap());

            let re_lib = Regex::new(r"lib(.+)\.a").unwrap();

            // Linking does not work if aria2 lib is listed after the libs it depends on
            println!("cargo:rustc-link-lib=aria2");

            for entry in lib_dir.read_dir().unwrap() {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if !path.is_file() { continue; }
                    if let Some(cap) = re_lib.captures(path.file_name().unwrap().to_str().unwrap()) {
                        let lib = cap.get(1).unwrap().as_str();
                        if lib != "aria2" {
                            println!("cargo:rustc-link-lib={}", lib);
                        }
                    }
                }
            }

            println!("cargo:rustc-link-lib=ws2_32");
            println!("cargo:rustc-link-lib=wsock32");
            println!("cargo:rustc-link-lib=gdi32");
            println!("cargo:rustc-link-lib=winmm");
            println!("cargo:rustc-link-lib=iphlpapi");
            println!("cargo:rustc-link-lib=psapi");
            println!("cargo:rustc-link-lib=crypt32");
            println!("cargo:rustc-link-lib=secur32");
            println!("cargo:rustc-link-lib=advapi32");

            // println!("cargo:rustc-link-search=native=/usr/x86_64-w64-mingw32/lib");
        }
    }

    bridge.compile("aria2_bridge");

    // Regen bridge if these files changes
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/aria2_bridge.cpp");
    println!("cargo:rerun-if-changed=include/aria2_bridge.hpp");
    println!("cargo:rerun-if-changed=include/DownloadHandleWrapper.hpp");
}
