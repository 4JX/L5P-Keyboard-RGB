use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn find_package(name: &str) -> Vec<PathBuf> {
    let library = vcpkg::find_package(name).expect("Failed to find package");
    println!("cargo:info={}", library.vcpkg_triplet); //TODO
    let lib_name = name.trim_start_matches("lib").to_string();
    println!("{}", format!("cargo:rustc-link-lib=static={}", lib_name));

    match (
        library.link_paths.as_slice(),
        library.include_paths.as_slice(),
    ) {
        ([link_search, ..], [include, ..]) => {
            println!(
                "{}",
                format!("cargo:rustc-link-search={}", link_search.display())
            );
            println!("{}", format!("cargo:include={}", include.display()));
        }
        _ => {
            panic!(
                "{}",
                if library.link_paths.is_empty() {
                    "link path not found"
                } else {
                    "include path not found"
                }
            )
        }
    }

    library.include_paths
}

fn generate_bindings(
    ffi_header: &Path,
    include_paths: &[PathBuf],
    ffi_rs: &Path,
    exact_file: &Path,
) {
    let mut b = bindgen::builder()
        .header(ffi_header.to_str().unwrap())
        .allowlist_type("^[vV].*")
        .allowlist_var("^[vV].*")
        .allowlist_function("^[vV].*")
        .rustified_enum("^v.*")
        .trust_clang_mangling(false)
        .layout_tests(false) // breaks 32/64-bit compat
        .generate_comments(false); // vpx comments have prefix /*!\

    for dir in include_paths {
        b = b.clang_arg(format!("-I{}", dir.display()));
    }

    b.generate().unwrap().write_to_file(ffi_rs).unwrap();
    fs::copy(ffi_rs, exact_file).ok(); // ignore failure
}

fn gen_vpx() {
    let includes = find_package("libvpx");
    let src_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let src_dir = Path::new(&src_dir);
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    let ffi_header = src_dir.join("vpx_ffi.h");
    println!("rerun-if-changed={}", ffi_header.display());
    for dir in &includes {
        println!("rerun-if-changed={}", dir.display());
    }

    let ffi_rs = out_dir.join("vpx_ffi.rs");
    let exact_file = src_dir.join("generated").join("vpx_ffi.rs");
    generate_bindings(&ffi_header, &includes, &ffi_rs, &exact_file);
}

fn main() {
    // note: all link symbol names in x86 (32-bit) are prefixed wth "_".
    // run "rustup show" to show current default toolchain, if it is stable-x86-pc-windows-msvc,
    // please install x64 toolchain by "rustup toolchain install stable-x86_64-pc-windows-msvc",
    // then set x64 to default by "rustup default stable-x86_64-pc-windows-msvc"
    let target = target_build_utils::TargetInfo::new();
    if target.unwrap().target_pointer_width() != "64" {
        // panic!("Only support 64bit system");
    }
    env::remove_var("CARGO_CFG_TARGET_FEATURE");
    env::set_var("CARGO_CFG_TARGET_FEATURE", "crt-static");

    find_package("libyuv");
    gen_vpx();

    // there is problem with cfg(target_os) in build.rs, so use our workaround
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "android" || target_os == "ios" {
        // nothing
    } else if cfg!(windows) {
        // The first choice is Windows because DXGI is amazing.
        println!("cargo:rustc-cfg=dxgi");
    } else if cfg!(target_os = "macos") {
        // Quartz is second because macOS is the (annoying) exception.
        println!("cargo:rustc-cfg=quartz");
    } else if cfg!(unix) {
        // On UNIX we pray that X11 (with XCB) is available.
        println!("cargo:rustc-cfg=x11");
    }
}
