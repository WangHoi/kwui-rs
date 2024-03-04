mod model;

use kwui::{Application, ScriptEngine};
use windows_dpi::enable_dpi;

fn main() {
    enable_dpi();

    let native = model::Model::new();
    let app = Application::new();
    ScriptEngine::load_file("D:/Projects/kwui-rs/examples/installer/assets/js/entry.js");

    native.start_install();

    app.exec();
}
