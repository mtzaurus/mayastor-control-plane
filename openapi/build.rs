use std::process::Command;

fn main() {
    let output = Command::new("bash")
        .args(&[
            "-c",
            "../scripts/rust/generate-openapi-bindings.sh --skip-md5-same --skip-git-diff",
        ])
        .output()
        .expect("failed to execute bash command");

    if !output.status.success() {
        panic!("openapi update failed: {:?}", output);
    }

    println!("cargo:rerun-if-changed=../nix/pkgs/openapi-generator");
    println!("cargo:rerun-if-changed=../control-plane/rest/openapi-specs");
    println!("cargo:rerun-if-changed=version.txt");
    // seems the internal timestamp is taken before build.rs runs, so we can't set this
    // directive against files created during the build of build.rs??
    // https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed
    // println!("cargo:rerun-if-changed=.");
}
