use embed_resource;
use kwui_cli;
use walkdir::WalkDir;

fn main() {
    pack_examples_resources();

    embed_resource::compile_for_examples("resources/windows/icon.rc", embed_resource::NONE);
}

fn pack_examples_resources() {
    let examples_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/examples");

    for entry in WalkDir::new(examples_dir)
        .min_depth(2)
        .max_depth(2)
        .into_iter()
        .filter_entry(|e| e.file_type().is_dir() && e.file_name() == "assets")
    {
        let entry = entry.unwrap();
        let base_dir = entry.path().parent().unwrap();
        let output_file = format!("{}/assets.ar", base_dir.display());
        let input_item = kwui_cli::PackInput::DirMapping {
            src: entry.path().to_string_lossy().to_string(),
            dst: "/".to_string(),
        };
        if let Err(_) = kwui_cli::pack(&output_file, &[input_item]) {
            println!(
                "cargo:warning=pack assets resource for {} failed",
                base_dir.file_stem().unwrap_or_default().to_string_lossy()
            );
        }
    }
}
