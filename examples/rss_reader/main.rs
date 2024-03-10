#![cfg_attr(all(not(test), not(debug_assertions)), windows_subsystem = "windows")]

mod rss_model;

use kwui::{Application, ScriptEngine};
use rss_model::Model;
use windows_dpi::enable_dpi;

fn main() {
    enable_dpi();

    let app = Application::new();

    if cfg!(debug_assertions) {
        app.set_resource_root_dir(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/rss_reader/assets"
        ));
    } else {
        const RES: &'static [u8] = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/rss_reader/assets.ar"
        ));
        app.set_resource_root_data(RES);
    }

    Model::init();

    ScriptEngine::load_file(":/js/entry.js");

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
