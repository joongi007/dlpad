//! Precompiled original artwork resources, no build-time external tools.
fn main() {
    println!("cargo:rerun-if-changed=assets/dlpad.res");
    println!("cargo:rerun-if-changed=assets/dlpad-icon.o");
    println!("cargo:rerun-if-changed=assets/dlpad.ico");
    println!("cargo:rerun-if-changed=assets/dlpad-installer.manifest");
    println!("cargo:rerun-if-changed=assets/dlpad-installer.res");
    println!("cargo:rerun-if-changed=assets/dlpad-installer.o");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").expect("target environment");
    let file = match target_env.as_str() {
        "msvc" => "dlpad.res",
        "gnu" => "dlpad-icon.o",
        _ => panic!("Windows icon resource target unsupported"),
    };
    let manifest =
        std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("manifest"));
    let resource = manifest.join("assets").join(file);
    for binary in [
        "dlpad",
        "dlpad-cpu",
        "dlpad-accessible",
        "dlpad-egui-spike",
        "dlpad-install",
    ] {
        let selected = if binary == "dlpad-install" {
            manifest.join("assets").join(match target_env.as_str() {
                "msvc" => "dlpad-installer.res",
                "gnu" => "dlpad-installer.o",
                _ => unreachable!(),
            })
        } else {
            resource.clone()
        };
        println!("cargo:rustc-link-arg-bin={binary}={}", selected.display());
    }
}
