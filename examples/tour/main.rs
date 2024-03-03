use kwui::{Application, ScriptEngine};
use windows_dpi::enable_dpi;

fn main() {
    enable_dpi();

    let app = Application::new();
    ScriptEngine::load_file("examples/tour/assets/entry.js");
    app.exec();
}
