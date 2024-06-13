use std::fs::File;
use std::path::PathBuf;
use flate2::Compression;
use flate2::write::GzEncoder;
use tar::Builder;
use crate::check_source_dir;

const KWUI_TEMPLATES: &'static str = "kwui-templates";

pub fn package(source_dir: &PathBuf, key: &str) -> anyhow::Result<()> {
    println!("Template release on source_dir [{}] with key {} ...", source_dir.display(), key);
    check_source_dir(source_dir)?;

    let cpp_android_template_dir = source_dir.join("kwui-sys/deps/kwui/cmake/AndroidPackaging.template");
    let template_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("templates/app");

    let out_filename = format!("{}-{}.tar.gz", KWUI_TEMPLATES, key);
    let tar_gz = File::create(&out_filename)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    tar.append_dir_all("app", template_dir)?;
    tar.append_dir_all("app/android", cpp_android_template_dir)?;
    tar.finish()?;

    Ok(())
}
