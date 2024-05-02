#![cfg_attr(all(not(test), not(debug_assertions)), windows_subsystem = "windows")]

mod install_model;

use install_model::Model;
use kwui::{Application, ScriptEngine};

#[cfg(target_os = "windows")]
use windows_dpi::enable_dpi;

#[kwui::main]
fn main() {
    #[cfg(target_os = "windows")]
    enable_dpi();

    let app = Application::new();
    if cfg!(debug_assertions) {
        app.set_resource_root_dir(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/installer/assets"
        ));
    } else {
        const RES: &'static [u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/installer/assets.ar"
        ));
        app.set_resource_root_data(RES);
    }

    ScriptEngine::load_file(":/js/entry.js");

    Model::init();
    Model::start_install();

    app.exec();

    Model::deinit();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        eprintln!("miao");
    }
}
