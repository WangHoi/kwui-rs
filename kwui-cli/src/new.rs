use std::{env, fs, io::{self, Error, ErrorKind, Read}, path::Path};
use std::fs::File;
use std::io::{Cursor, Write};
use std::path::PathBuf;
use clap::builder::OsStr;
use flate2::read::GzDecoder;
use itertools::Itertools;
use tar::Archive;
use regex::Regex;
use globmatch;
use fs_extra;
use path_absolutize::*;

pub fn new_project(with_kwui: Option<PathBuf>, output_dir: &PathBuf, prj_type: &str, crate_name: &str) -> anyhow::Result<()> {
    println!("CREATE PROJECT IN [{}]", output_dir.display());
    std::fs::create_dir_all(output_dir)?;

    apply_template(output_dir, prj_type, crate_name)?;

    if let Some(kwui_dir) = with_kwui {
        patch_kwui(&kwui_dir, output_dir)?;
    }

    android_copy_sdk_licenses(output_dir)?;
    android_copy_cxx_stl_library(output_dir)?;

    println!("CREATE {} PROJECT [{}] DONE", prj_type, crate_name);
    Ok(())
}

fn patch_kwui(kwui_dir: &PathBuf, output_dir: &PathBuf) -> anyhow::Result<()> {
    let kwui_dir = kwui_dir.absolutize().unwrap().display().to_string().replace("\\", "/");
    println!("PATCHING kwui crate to [{}]", kwui_dir);
    let mut f = File::options().append(true).open(output_dir.join("Cargo.toml"))?;
    let content = format!("\n\
[patch.crates-io]\n\
kwui = {{ path = \"{kwui_dir}\" }}\n\
kwui-sys = {{ path = \"{kwui_dir}/kwui-sys\" }}\n\
kwui-cli = {{ path = \"{kwui_dir}/kwui-cli\" }}\n\
");
    f.write_all(&content.into_bytes())?;
    Ok(())
}

pub fn apply_template(output_dir: &PathBuf, prj_type: &str, crate_name: &str) -> anyhow::Result<()> {
    let templates_url = kwui_templates_url();
    println!("USING TEMPLATES URL [{}]", templates_url);
    let template = download(&templates_url)?;

    let mut tar = GzDecoder::new(Cursor::new(template));
    let mut tar = Archive::new(tar);

    let prefix = String::from(prj_type) + "/";
    for entry in tar.entries()? {
        let mut entry = entry?;
        let src_path = entry.path()?.to_path_buf();

        if let Ok(src_path) = src_path.strip_prefix(&prefix) {
            let need_patch = src_path
                .extension()
                .filter(|ext| ext.to_string_lossy() == "in")
                .is_some();
            if need_patch {
                let mut content = String::new();
                entry.read_to_string(&mut content)?;

                let content = configure_file(&src_path, content, crate_name, "0.1.0");

                // strip ".in" postfix
                let mut dst_path = output_dir.join(src_path);
                dst_path.set_extension("");

                fs::write(&dst_path, content)?;
            } else {
                let dst_path = output_dir.join(src_path);
                entry.unpack(&dst_path);
            }
        }
    }
    Ok(())
}

fn configure_file(filename: &Path, content: String, crate_name: &str, crate_version: &str) -> String {
    let crate_id = String::from("proj.kwui.") + crate_name;
    let crate_version_code = "1000";

    let content = content.replace("@KWUI_TEMPLATES_TAG@", env!("TAG"));
    let content = content.replace("@ANDROID_APPLICATION_NAME@", crate_name);
    let content = content.replace("@ANDROID_APPLICATION_ID@", &crate_id);
    let content = content.replace("@ANDROID_APPLICATION_VERSIONNAME@", crate_version);
    let content = content.replace("@ANDROID_APPLICATION_VERSIONCODE@", crate_version_code);
    let content = content.replace("@ANDROID_PACKAGE_NAME@", crate_name);
    let content = content.replace("@ANDROID_PACKAGE_ID@", &crate_id);
    let content = content.replace("@ANDROID_PACKAGE_VERSIONNAME@", crate_version);
    let content = content.replace("@ANDROID_PACKAGE_VERSIONCODE@", crate_version_code);
    let content = content.replace("@ANDROID_ADDITIONAL_PARAMS@", "");
    let content = content.replace("@ANDROID_ASSETS_DIR@", "../assets");
    let content = content.replace("@JAVA_HOME@", &java_home());
    let content = content.replace("@CMAKE_ANDROID_SDK@", &android_sdk_home().display().to_string());

    let re = Regex::new(r"(@[a-zA-Z_]+@)").unwrap();
    for (_, [placeholder]) in re.captures_iter(&content).map(|c| c.extract()) {
        eprintln!("error: {}: unhandled template placeholder {}", filename.display(), placeholder);
    }

    content
}

/// Download a file from the given URL and return the data.
pub fn download(url: impl AsRef<str>) -> io::Result<Vec<u8>> {
    let url = url.as_ref();

    // `file` URL, empty hostname, absolute path
    if let Some(file_path) = url.strip_prefix("file://") {
        return fs::read(Path::new(file_path));
    }

    // `file` URLs with non-empty hostname or relative paths are unsupported.
    if url.starts_with("file:") {
        eprintln!("Unsupported file: URL {}", url);
        return Err(Error::from(ErrorKind::Unsupported));
    }
    let resp = std::process::Command::new("curl")
        // follow redirects
        .arg("-L")
        // fail fast with no "error pages" output. more of a hint though, so we might still get error on stdout.
        // so make sure to check the actual status returned.
        .arg("-f")
        // silent. no progress or error messages. only pure "response data"
        .arg("-s")
        .arg(url)
        .output();
    match resp {
        Ok(out) => {
            // ideally, we would redirect response to a file directly, but lets take it one step at a time.
            let result = out.stdout;
            if out.status.success() {
                Ok(result)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "curl error code: {:?}\ncurl stderr: {:?}",
                        out.status.code(),
                        std::str::from_utf8(&out.stderr)
                    ),
                ))
            }
        }
        Err(e) => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("curl command error : {e:#?}"),
        )),
    }
}

pub fn kwui_templates_url() -> String {
    let url = std::env::var("KWUI_TEMPLATES_URL")
        .unwrap_or("https://github.com/wanghoi/kwui-binaries/releases/download/{tag}/kwui-templates-{key}.tar.gz"
            .into());
    url.replace("{tag}", env!("TAG"))
        .replace("{key}", env!("KEY"))
}

fn java_home() -> String {
    std::env::var("JAVA_HOME").expect("JAVA_HOME environment variable not set")
}

fn android_sdk_home() -> PathBuf {
    std::env::var("ANDROID_HOME").expect("ANDROID_HOME environment variable not set").into()
}

fn android_copy_sdk_licenses(output_dir: &PathBuf) -> anyhow::Result<()> {
    let copy_src_dir = android_sdk_home().join("licenses");

    if copy_src_dir.is_dir() {
        println!("COPY ANDROID LICENSES ");
        let copy_dest_dir = output_dir.join("android");
        std::fs::create_dir_all(&copy_dest_dir).expect("create android licenses directory failed");
        let copy_opts = fs_extra::dir::CopyOptions::new().overwrite(true);
        fs_extra::copy_items(&[copy_src_dir], copy_dest_dir, &copy_opts)?;
    } else {
        println!("cargo::warning=WARN: copy android licenses dir [{}] not found.", copy_src_dir.display());
    }

    Ok(())
}

fn android_copy_cxx_stl_library(output_dir: &PathBuf) -> anyhow::Result<()> {
    println!("COPY ANDROID cxx_shared LIBRARY");

    let src_file = find_library("**/aarch64-linux-android/libc++_shared.so");
    let copy_dest_dir = output_dir.join("android/app/src/main/jniLibs/arm64-v8a");
    std::fs::create_dir_all(&copy_dest_dir).expect("create jniLibs directory failed");
    std::fs::copy(&src_file, &copy_dest_dir.join("libc++_shared.so"))
        .expect("copy libc++_shared.so failed");

    Ok(())
}

fn android_ndk_home() -> PathBuf {
    std::env::var("ANDROID_NDK_HOME")
        .expect("environment variable \"ANDROID_NDK_HOME\" not set")
        .into()
}

fn manifest_dir() -> PathBuf {
    std::env::var("CARGO_MANIFEST_DIR").unwrap().into()
}

fn find_library(filepath: &str) -> PathBuf {
    let b = globmatch::Builder::new(filepath)
        .build(android_ndk_home())
        .unwrap();
    if let Some(Ok(p)) = b.into_iter().next() {
        return p;
    } else {
        panic!("find_library {} failed", filepath)
    }
}
