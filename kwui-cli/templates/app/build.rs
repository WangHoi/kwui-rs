use std::path::PathBuf;
use build_target;
#[allow(unused_imports)]
use embed_resource;
use kwui_cli;

fn main() {
    pack_resources();
    // embed_resource::compile_for_examples("resources/windows/icon.rc", embed_resource::NONE);

    if let Ok(build_target::Os::Android) = build_target::target_os() {
        android_copy_support_files();
    }
    println!("cargo::metadata=sys_root={}", std::env::var("DEP_KWUI_SYS_ROOT").unwrap());
}

fn pack_resources() {
    println!("PACK RESOURCES");
    let asset_dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/assets"));

    let base_dir = asset_dir.parent().unwrap();
    let output_file = format!("{}/assets.ar", base_dir.display());
    let input_item = kwui_cli::PackInput::DirMapping {
        src: asset_dir.to_string_lossy().to_string(),
        dst: "/".to_string(),
    };
    if let Err(_) = kwui_cli::pack(&output_file, &[input_item]) {
        println!(
            "cargo:warning=pack assets resource for {} failed",
            base_dir.file_stem().unwrap_or_default().to_string_lossy()
        );
    }
}

fn android_copy_support_files() {
    android_copy_kwui_library();
}

fn android_copy_kwui_library() {
    println!("COPY KWUI ANDROID LIBRARY");
    let copy_dest_dir = manifest_dir().join("android/app/src/main/jniLibs/arm64-v8a");
    std::fs::create_dir_all(&copy_dest_dir).expect("create jniLibs directory failed");

    let kwui_src_dir = std::env::var("DEP_KWUI_SYS_ROOT").unwrap();
    std::fs::copy(
        format!("{}/lib/libkwui.so", kwui_src_dir),
        copy_dest_dir.join("libkwui.so"),
    ).expect("copy libkwui.so failed");

    println!("cargo::rerun-if-env-changed=DEP_KWUI_SYS_ROOT");
    println!("cargo::rerun-if-changed={}/lib/libkwui.so", kwui_src_dir);
    println!("cargo::rerun-if-changed={}/libkwui.so", copy_dest_dir.display());
    println!("cargo::rerun-if-changed=src");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");
}

fn manifest_dir() -> PathBuf {
    std::env::var("CARGO_MANIFEST_DIR").unwrap().into()
}
