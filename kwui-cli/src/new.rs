use std::{
    fs,
    io::{self, Error, ErrorKind, Read},
    path::Path,
};
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;
use clap::builder::OsStr;
use flate2::read::GzDecoder;
use itertools::Itertools;
use tar::Archive;
use regex::Regex;

pub fn new_project(output_dir: &PathBuf, prj_type: &str, crate_name: &str) -> anyhow::Result<()> {
    println!("CREATE PROJECT IN {}", output_dir.display());
    std::fs::create_dir_all(output_dir)?;

    let templates_url = kwui_templates_url();
    println!("USING TEMPLATES URL {}", templates_url);
    let template = download(&templates_url)?;
    println!("USING TEMPLATES URL {}", template.len());

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

    let content = content.replace("@ANDROID_APPLICATION_NAME@", crate_name);
    let content = content.replace("@ANDROID_APPLICATION_ID@", &crate_id);
    let content = content.replace("@ANDROID_APPLICATION_VERSIONNAME@", crate_version);
    let content = content.replace("@ANDROID_APPLICATION_VERSIONCODE@", crate_version_code);
    let content = content.replace("@ANDROID_PACKAGE_NAME@", crate_name);
    let content = content.replace("@ANDROID_PACKAGE_ID@", &crate_id);
    let content = content.replace("@ANDROID_PACKAGE_VERSIONNAME@", crate_version);
    let content = content.replace("@ANDROID_PACKAGE_VERSIONCODE@", crate_version_code);
    let content = content.replace("@ANDROID_ADDITIONAL_PARAMS@", "");
    let content = content.replace("@JAVA_HOME@", &java_home());
    let content = content.replace("@CMAKE_ANDROID_SDK@", &android_sdk());

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
    const KWUI_TEMPLATES_TAG: &'static str = "0.1.0";
    const KWUI_TEMPLATES_KEY: &'static str = "80cfc710d93d4b557730";

    let url = std::env::var("KWUI_TEMPLATES_URL")
        .unwrap_or("https://github.com/wanghoi/kwui-templates/releases/download/{tag}/kwui-binaries-{key}.tar.gz"
            .into());
    url.replace("{tag}", KWUI_TEMPLATES_TAG)
        .replace("{key}", KWUI_TEMPLATES_KEY)
}

fn java_home() -> String {
    std::env::var("JAVA_HOME").expect("JAVA_HOME environment variable not set")
}

fn android_sdk() -> String {
    std::env::var("ANDROID_HOME").expect("ANDROID_HOME environment variable not set")
}