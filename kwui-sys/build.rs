mod build_support;

use bindgen;
use build_support::{
    binaries_config,
    cargo::{self, Target},
    features, kwui,
};
use std::io;

fn main() -> Result<(), io::Error> {
    if env::is_docs_rs_build() {
        println!("DETECTED DOCS_RS BUILD");
        return fake_bindings();
    }

    let skia_debug = env::is_kwui_debug();
    let features = features::Features::default();
    let binaries_config =
        binaries_config::BinariesConfiguration::from_features(&features, skia_debug);

    // Check external kwui cpp source directory
    if let Some(source_dir) = env::source_dir() {
        if let Some(search_path) = env::kwui_lib_search_path() {
            println!("STARTING BIND AGAINST SYSTEM KWUI");

            binaries_config.import(&search_path, false).unwrap();

            // let definitions = kwui_bindgen::definitions::from_env();
            generate_bindings(
                // &features,
                // definitions,
                &binaries_config,
                &source_dir,
                cargo::target(),
                // None,
            );
        } else {
            println!("STARTING OFFLINE BUILD");

            let final_build_configuration = build_from_source(
                features.clone(),
                &binaries_config,
                &source_dir,
                skia_debug,
                true,
            );

            // let definitions = skia_bindgen::definitions::from_ninja_features(
            //     &features,
            //     final_build_configuration.use_system_libraries,
            //     &binaries_config.output_directory,
            // );
            generate_bindings(
                // &features,
                // definitions,
                &binaries_config,
                &source_dir,
                final_build_configuration.target,
                // final_build_configuration
                //     .sysroot
                //     .as_ref()
                //     .map(AsRef::as_ref),
            );
        }
    } else {
        //
        // is the download of prebuilt binaries possible?
        //

        let build_kwui = build_support::binary_cache::try_prepare_download(&binaries_config);

        //
        // full build?
        //

        if build_kwui {
            println!("STARTING A FULL BUILD");

            let source_dir = std::env::current_dir().unwrap().join("deps/kwui");
            let final_build_configuration = build_from_source(
                features.clone(),
                &binaries_config,
                &source_dir,
                skia_debug,
                false,
            );

            // let definitions = skia_bindgen::definitions::from_ninja_features(
            //     &features,
            //     final_build_configuration.use_system_libraries,
            //     &binaries_config.output_directory,
            // );
            generate_bindings(
                // &features,
                // definitions,
                &binaries_config,
                &source_dir,
                final_build_configuration.target,
                // final_build_configuration
                //     .sysroot
                //     .as_ref()
                //     .map(AsRef::as_ref),
            );
        }
    };

    binaries_config.commit_to_cargo();

    if let Some(staging_directory) = build_support::binary_cache::should_export() {
        build_support::binary_cache::publish(&binaries_config, &staging_directory);
    }

    println!("cargo:root={}", std::env::var("OUT_DIR").unwrap());

    Ok(())
}

fn build_from_source(
    features: features::Features,
    binaries_config: &binaries_config::BinariesConfiguration,
    kwui_source_dir: &std::path::Path,
    kwui_debug: bool,
    offline: bool,
) -> kwui::FinalBuildConfiguration {
    let build_config = kwui::BuildConfiguration::from_features(features, kwui_debug);
    let final_configuration = kwui::FinalBuildConfiguration::from_build_configuration(
        &build_config,
        kwui::env::use_system_libraries(),
        kwui_source_dir,
    );

    kwui::build(
        &final_configuration,
        binaries_config,
        kwui::env::ninja_command(),
        kwui::env::cmake_command(),
        offline,
    );

    final_configuration
}

fn generate_bindings(
    // features: &features::Features,
    // definitions: Vec<skia_bindgen::Definition>,
    binaries_config: &binaries_config::BinariesConfiguration,
    _kwui_source_dir: &std::path::Path,
    target: Target,
    // sysroot: Option<&str>,
) {
    let clang_args: Vec<String> = match target.system.as_str() {
        "android" => {
            let android_ndk_home = std::env::var("ANDROID_NDK_HOME").unwrap();
            //panic!("{}", format!("-I{}/toolchains/llvm/prebuilt/windows-x86_64/lib/clang/17.0.2/include", android_ndk_home));
            vec![
                "-DKWUI_SHARED_LIBRARY=1".to_string(),
                format!("--sysroot={}/toolchains/llvm/prebuilt/windows-x86_64/sysroot", android_ndk_home),
                format!("-I{}/toolchains/llvm/prebuilt/windows-x86_64/lib/clang/18/include", android_ndk_home),
            ]
        }
        _ => vec!["-DKWUI_STATIC_LIBRARY=1".to_string()],
    };
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(format!("{}/include/kwui_capi.h", binaries_config.output_directory.display()))
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
    /*
    // Emit the ninja definitions, to help debug build consistency.
    skia_bindgen::definitions::save_definitions(&definitions, &binaries_config.output_directory)
        .expect("failed to write Skia defines");

    let bindings_config = skia_bindgen::Configuration::new(features, definitions, skia_source_dir);
    skia_bindgen::generate_bindings(
        &bindings_config,
        &binaries_config.output_directory,
        target,
        sysroot,
    );
    */
}

/// On docs.rs, rustdoc runs inside a container with no networking, so copy a pre-generated
/// `bindings.rs` file.
fn fake_bindings() -> Result<(), io::Error> {
    Ok(())
    /*
    println!("COPYING bindings_docs.rs to OUT_DIR/skia/bindings.rs");
    let bindings_target = cargo::output_directory()
        .join(binaries_config::SKIA_OUTPUT_DIR)
        .join("bindings.rs");
    fs::copy("bindings_docs.rs", bindings_target).map(|_| ())
    */
}

/// Environment variables used by this build script.
mod env {
    use crate::build_support::cargo;
    use std::path::PathBuf;

    /// The path to the Skia source directory.
    pub fn source_dir() -> Option<PathBuf> {
        cargo::env_var("KWUI_SOURCE_DIR").map(PathBuf::from)
    }

    /// The path to where a pre-built Skia library can be found.
    pub fn kwui_lib_search_path() -> Option<PathBuf> {
        cargo::env_var("KWUI_LIBRARY_SEARCH_PATH").map(PathBuf::from)
    }

    pub fn is_kwui_debug() -> bool {
        matches!(cargo::env_var("KWUI_DEBUG"), Some(v) if v != "0")
    }

    pub fn is_docs_rs_build() -> bool {
        matches!(cargo::env_var("DOCS_RS"), Some(v) if v != "0")
    }
}
/*
fn old_main() {
    let cmake_project_dir = "deps/kwui";
    // println!("cargo:rerun-if-changed={}/cmake", cmake_project_dir);
    // println!("cargo:rerun-if-changed={}/src", cmake_project_dir);

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
        }
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
*/
