use std::{env, path::PathBuf};

use libbpf_cargo::SkeletonBuilder;

const SRC: &str = "third_party/NetworkManager/src/core/bpf/clat.bpf.c";

fn main() {
    let mut out =
        PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set in build script"));
    out.push("clat.skel.rs");

    SkeletonBuilder::new()
        .source(SRC)
        .clang_args(["-I", "include", "-D", "NKXLAT"])
        .build_and_generate(&out)
        .expect("bpf compilation failed");

    println!("cargo:rerun-if-changed={SRC}");
}
