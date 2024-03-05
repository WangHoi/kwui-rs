mod install_model;

use kwui::{Application, ScriptEngine};
use install_model::Model;
use windows_dpi::enable_dpi;

fn main() {
    enable_dpi();

    let app = Application::new();
    app.set_resource_root_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/installer/assets"));
    ScriptEngine::load_file(":/js/entry.js");
    
    Model::init();
    Model::start_install();

    app.exec();

    Model::deinit();
    
}
