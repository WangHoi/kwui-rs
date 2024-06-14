#![allow(unused, dead_code)]

pub mod file_format;
pub mod packager;
pub mod binary_release;
pub mod template_release;
pub mod new;
pub mod build;
pub mod run;

use itertools::Itertools;
use path_clean;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use cargo_toml::Manifest;

pub use packager::{list, unpack, PackItem};

pub enum PackInput {
    SourceFile { src: String },
    SourceDir { src: String },
    FileMapping { src: String, dst: String },
    DirMapping { src: String, dst: String },
}

pub fn pack(output_file: &str, input_list: &[PackInput]) -> std::io::Result<()> {
    let mut file_items = Vec::new();
    let mut dir_items = Vec::new();
    for input in input_list.into_iter() {
        match input {
            PackInput::SourceFile { src } => {
                let src = path_clean::clean(&src);
                let src_file_name = src
                    .file_name()
                    .ok_or_else(|| {
                        std::io::Error::other(format!("invalid source filename {}", src.to_string_lossy()))
                    })?
                    .to_string_lossy()
                    .to_string();
                let dst = String::from("/") + src_file_name.as_str();
                file_items.push(PackItem {
                    src: src.to_string_lossy().into(),
                    dst,
                });
            }
            PackInput::SourceDir { src } => {
                let src = path_clean::clean(&src);
                let src_file_name = src
                    .file_name()
                    .ok_or_else(|| {
                        std::io::Error::other(format!("invalid source directory {}", src.to_string_lossy()))
                    })?
                    .to_string_lossy()
                    .to_string();
                let dst = String::from("/") + src_file_name.as_str();

                if let Ok((sub_file_items, sub_dir_items)) = scan_dir(src.as_ref(), &dst) {
                    for item in sub_file_items.into_iter() {
                        file_items.push(item);
                    }
                    for item in sub_dir_items.into_iter() {
                        dir_items.push(item);
                    }
                }
            }
            PackInput::FileMapping { src, dst } => {
                let src = path_clean::clean(&src);
                let src_file_name = src
                    .file_name()
                    .ok_or_else(|| {
                        std::io::Error::other(format!("invalid source filename {}", src.to_string_lossy()))
                    })?
                    .to_string_lossy()
                    .to_string();
                let dst = if dst.ends_with("/") {
                    dst.to_string() + src_file_name.as_str()
                } else {
                    dst.to_string()
                };
                file_items.push(PackItem {
                    src: src.to_string_lossy().into(),
                    dst,
                });
            }
            PackInput::DirMapping { src, dst } => {
                let src = path_clean::clean(&src);
                let dst = if dst.ends_with("/") {
                    dst[..(dst.len() - 1)].to_string()
                } else {
                    dst.to_string()
                };

                if let Ok((sub_file_items, sub_dir_items)) = scan_dir(src.as_ref(), &dst) {
                    for item in sub_file_items.into_iter() {
                        file_items.push(item);
                    }
                    for item in sub_dir_items.into_iter() {
                        dir_items.push(item);
                    }
                }
            }
        }
    }
    for f_item in file_items.iter() {
        if let Some(idx) = f_item.dst.rfind('/') {
            let dir = &f_item.dst[..(idx + 1)];
            // println!("add [{}]", dir);
            dir_items.push(dir.to_string());
        }
    }
    dir_items.push("/".to_string());
    dir_items.sort_by_key(|x| x.to_lowercase());
    dir_items = dir_items.into_iter().dedup().collect();
    crate::packager::pack(output_file, file_items, dir_items).map_err(|e| std::io::Error::other(e))
}

fn scan_dir(dir: &Path, dst: &str) -> anyhow::Result<(Vec<PackItem>, Vec<String>)> {
    let mut file_items = Vec::new();
    let mut dir_items = Vec::new();
    for entry in walkdir::WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let full_path = entry.path().to_string_lossy();
        let f_components = entry
            .path()
            .components()
            .map(|x| x.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        let f_components = &f_components[(f_components.len() - entry.depth())..];
        let dst = format!("{}/{}", dst, f_components.join("/"));

        let meta = entry.metadata()?;
        if meta.is_dir() {
            let full_path = full_path + "/";
            let dst = if dst.ends_with('/') { dst } else { dst + "/" };
            println!("scan_dir, add dir [{}]:[{}]", full_path, dst);
            dir_items.push(dst);
        } else if meta.is_file() {
            println!("scan_dir, add file [{}]:[{}]", full_path, dst);
            file_items.push(PackItem {
                src: full_path.to_string(),
                dst,
            });
        }
    }
    Ok((file_items, dir_items))
}

pub fn check_source_dir(source_dir: &PathBuf) -> anyhow::Result<()> {
    let manifest = Manifest::from_path(source_dir.join("Cargo.toml"))
        .map_err(|e| {
            eprintln!("Load Cargo.toml error: {}", e);
            e
        })?;
    if let Some(ws) = manifest.workspace {
        if ws.members.contains(&String::from("kwui-sys")) {
            return Ok(());
        }
    }
    anyhow::bail!("Invalid source_dir {}", source_dir.display())
}

pub fn git_half_hash() -> Option<String> {
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

pub fn cargo_package_name(source_dir: &PathBuf) -> anyhow::Result<String> {
    let manifest = Manifest::from_path(source_dir.join("Cargo.toml"))
        .map_err(|e| {
            eprintln!("Load Cargo.toml error: {}", e);
            e
        })?;
    if let Some(pkg) = manifest.package {
        Ok(pkg.name.clone())
    } else {
        anyhow::bail!("Invalid source_dir {}", source_dir.display())
    }
}
