use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use libbpf_cargo::SkeletonBuilder;

const SRC: &str = "third_party/NetworkManager/src/core/bpf/clat.bpf.c";

fn main() {
    let mut out =
        PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set in build script"));
    out.push("clat.skel.rs");

    let arch = env::var("CARGO_CFG_TARGET_ARCH")
        .expect("CARGO_CFG_TARGET_ARCH must be set in build script");

    SkeletonBuilder::new()
        .source(SRC)
        .clang_args([
            OsStr::new("-I"),
            Path::new("third_party/vmlinux.h/include")
                .join(arch)
                .as_os_str(),
            OsStr::new("-D"),
            OsStr::new("NKXLAT"),
        ])
        .build_and_generate(&out)
        .expect("bpf compilation failed");

    println!("cargo:rerun-if-changed={SRC}");
}
