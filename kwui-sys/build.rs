use cmake::{self, Config};
use bindgen;

fn main() {
    let cmake_project_dir = "deps/kwui";
    println!("cargo:rerun-if-changed={}/cmake", cmake_project_dir);
    println!("cargo:rerun-if-changed={}/src", cmake_project_dir);
    let dst = Config::new(cmake_project_dir)
        .profile("Release")
        .define("CMAKE_CONFIGURATION_TYPES", "Release")
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

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(format!("{}/include/kwui_capi.h", dst.display()))
        .allowlist_function("kwui_.*")
        .layout_tests(false)
        .clang_args([
            "-DKWUI_STATIC_LIBRARY=1",
            ])
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
