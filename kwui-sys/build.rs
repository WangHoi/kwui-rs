use bindgen;
use cmake::{self, Config};
use std::path::PathBuf;
use build_target;

fn main() {
    let cmake_project_dir = "deps/kwui";
    println!("cargo:rerun-if-changed={}/cmake", cmake_project_dir);
    println!("cargo:rerun-if-changed={}/src", cmake_project_dir);

    let target_os = build_target::target_os().unwrap();
    let dst = match target_os {
        build_target::Os::Android => cmake_config_android(cmake_project_dir),
        _ => cmake_config_windows(cmake_project_dir),
    };

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let clang_args: Vec<String> = match target_os {
        build_target::Os::Android => {
            let android_ndk_home = std::env::var("ANDROID_NDK_HOME").unwrap();
            //panic!("{}", format!("-I{}/toolchains/llvm/prebuilt/windows-x86_64/lib/clang/17.0.2/include", android_ndk_home));
            vec![
                "-DKWUI_SHARED_LIBRARY=1".to_string(),
                format!("--sysroot={}/toolchains/llvm/prebuilt/windows-x86_64/sysroot", android_ndk_home),
                format!("-I{}/toolchains/llvm/prebuilt/windows-x86_64/lib/clang/18/include", android_ndk_home),
            ]
        },
        _ => vec!["-DKWUI_STATIC_LIBRARY=1".to_string()],
    };
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(format!("{}/include/kwui_capi.h", dst.display()))
        .allowlist_function("kwui_.*")
        .layout_tests(false)
        .clang_args(clang_args)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

#[cfg(target_feature = "crt-static")]
fn crt_static() -> bool {
    true
}

#[cfg(not(target_feature = "crt-static"))]
fn crt_static() -> bool {
    false
}

fn cmake_config_windows(cmake_project_dir: &str) -> PathBuf {
    let dst = Config::new(cmake_project_dir)
        .profile("Release")
        .define("CMAKE_CONFIGURATION_TYPES", "Release")
        .define("CRT_STATIC", if crt_static() { "ON" } else { "OFF" })
        .define("BUILD_TESTS", "OFF")
        .define("BUILD_EXAMPLES", "OFF")
        .generator("Ninja")
        .build();
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=kwui_static");
    println!("cargo:rustc-link-lib=User32");
    println!("cargo:rustc-link-lib=Gdi32");
    println!("cargo:rustc-link-lib=Imm32");
    println!("cargo:rustc-link-lib=d2d1");
    println!("cargo:rustc-link-lib=dwrite");
    println!("cargo:rustc-link-lib=Windowscodecs");
    println!("cargo:root={}", dst.display());
    dst
}

fn cmake_config_android(cmake_project_dir: &str) -> PathBuf {
    let android_ndk_home = std::env::var("ANDROID_NDK_HOME").unwrap();
    let dst = Config::new(cmake_project_dir)
        .profile("Release")
        .define("CMAKE_CONFIGURATION_TYPES", "Release")
        .define("CMAKE_SYSTEM_NAME", "Android")
        .define("CMAKE_SYSTEM_VERSION", "30")
        .define("CMAKE_ANDROID_ARCH_ABI", "arm64-v8a")
        .define("CMAKE_ANDROID_NDK", &android_ndk_home)
        .define("CMAKE_ANDROID_STL_TYPE", "c++_shared")
        .define("BUILD_SHARED_LIBS", "ON")
        .define("CRT_STATIC", "OFF")
        .define("BUILD_TESTS", "OFF")
        .generator("Ninja")
        .build();
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=dylib=kwui");
    println!("cargo:root={}", dst.display());
    dst
}
