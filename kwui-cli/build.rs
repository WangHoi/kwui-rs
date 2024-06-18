use std::{fs, io};
use std::process::{Command, Stdio};

fn main() {
    println!("cargo::rustc-env=TAG={}", std::env::var("CARGO_PKG_VERSION").unwrap());
    println!("cargo::rustc-env=KEY={}", kwui_templates_key());
}

fn kwui_templates_key() -> String {
    if let Ok(hash) = crate_repository_hash() {
        trim_hash(&hash)
    } else {
        half_hash().expect("Error: templates key not found.")
    }
}

// If we are building from within a crate, return the full commit hash
// of the repository the crate was packaged from.
fn crate_repository_hash() -> io::Result<String> {
    let vcs_info = fs::read_to_string(".cargo_vcs_info.json")?;
    let value: serde_json::Value = serde_json::from_str(&vcs_info)?;
    let git = value.get("git").expect("failed to get 'git' property");
    let sha1 = git.get("sha1").expect("failed to get 'sha1' property");
    Ok(sha1.as_str().unwrap().into())
}

const HALF_HASH_LENGTH: usize = 20;

fn trim_hash(hash: &str) -> String
{
    hash[..HALF_HASH_LENGTH].into()
}

fn half_hash() -> Option<String> {
    let mut cmd = Command::new("git");
    cmd.arg("rev-parse").arg("--short=20");
    let output = cmd.arg("HEAD").stderr(Stdio::inherit()).output().ok()?;
    if output.status.code() != Some(0) {
        None
    } else {
        // need to trim the string to remove newlines at the end.
        Some(String::from_utf8(output.stdout).unwrap().trim().to_string())
    }
}
