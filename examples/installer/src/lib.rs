mod install_model;

use install_model::Model;
use kwui::{Application, ScriptEngine};

#[cfg(target_os = "windows")]
use windows_dpi;

use kwui;

pub fn entry() {
    #[cfg(target_os = "windows")]
    windows_dpi::enable_dpi();

    let app = Application::new();
    if cfg!(all(target_os = "windows", debug_assertions)) {
        app.set_resource_root_dir(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets"
        ));
    } else {
        const RES: &'static [u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets.ar"
        ));
        app.set_resource_root_data(RES);
    }

    ScriptEngine::load_file(":/js/entry.js");

    Model::init();
    Model::start_install();

    app.exec();

    Model::deinit();
}

#[cfg(target_os = "android")]
#[no_mangle]
pub unsafe extern "C" fn kwui_main(_argc: std::os::raw::c_int, _argv: std::os::raw::c_char) -> std::os::raw::c_int {
    entry();
    0
}
