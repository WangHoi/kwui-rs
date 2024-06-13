#![allow(unused, dead_code)]

use anyhow;
use clap::{Parser, Subcommand};
use itertools::Itertools;
use kwui_cli::{self, git_half_hash, PackInput, PackItem};
use path_clean;
use std::{
    fmt::format,
    path::{Path, PathBuf},
};
use walkdir;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    PackArchive {
        output_file: String,

        #[arg(name = "SRC_FILE|SRC_DIR|SRC_FILE:DST_FILE|SRC_DIR:DST_DIR")]
        file_dir_mappings: Vec<String>,
    },
    UnpackArchive {
        input_file: String,
        target_dir: Option<String>,
    },
    ListArchive {
        input_file: String,
    },
    BinaryRelease {
        source_dir: PathBuf,
    },
    TemplateRelease {
        #[arg(long)]
        key: Option<String>,

        source_dir: PathBuf,
    },
    New {
        project_name: String,
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let current_dir = std::env::current_dir().unwrap();
    match cli.command {
        Commands::PackArchive {
            output_file,
            file_dir_mappings,
        } => {
            let mut inputs: Vec<PackInput> = Vec::new();
            for p in file_dir_mappings {
                let segs = if let Ok(_) = std::fs::metadata(&p) {
                    vec![p.to_string()]
                } else if let Some(idx) = p.rfind(':') {
                    let s1 = p[..idx].to_string();
                    let s2 = p[(idx + 1)..].to_string();
                    vec![s1, s2]
                } else {
                    vec![p.to_string()]
                };
                if segs.is_empty() {
                    continue;
                } else {
                    let src = path_clean::clean(&segs[0]);
                    println!("clean {}=>{}", segs[0], src.to_string_lossy());
                    let dst = if segs.len() >= 2 {
                        let dst = segs[1].to_string();
                        let dst_segs = dst
                            .split(&['/', '\\'])
                            .filter(|&x| !x.is_empty())
                            .collect::<Vec<_>>();
                        let new_dst = dst_segs.join("/");
                        if dst.ends_with("/") {
                            if new_dst.is_empty() {
                                String::from("/")
                            } else {
                                String::from("/") + new_dst.as_str() + "/"
                            }
                        } else if new_dst.is_empty() {
                            String::from("")
                        } else {
                            String::from("/") + new_dst.as_str()
                        }
                    } else {
                        String::from("")
                    };
                    let src_file_name = src.file_name().unwrap().to_string_lossy().to_string();
                    if src.is_file() {
                        if dst.is_empty() {
                            inputs.push(PackInput::SourceFile {
                                src: src.to_string_lossy().into(),
                            });
                        } else {
                            inputs.push(PackInput::FileMapping {
                                src: src.to_string_lossy().into(),
                                dst,
                            });
                        };
                    } else if src.is_dir() {
                        if dst.is_empty() {
                            inputs.push(PackInput::SourceDir {
                                src: src.to_string_lossy().into(),
                            });
                        } else {
                            inputs.push(PackInput::DirMapping {
                                src: src.to_string_lossy().into(),
                                dst,
                            });
                        };
                    }
                }
            }

            kwui_cli::pack(&output_file, &inputs)?;
        }
        Commands::UnpackArchive {
            input_file,
            target_dir,
        } => {
            let target_dir = target_dir.unwrap_or(current_dir.to_string_lossy().to_string());
            let target_dir = std::fs::canonicalize(&target_dir)?;
            kwui_cli::packager::unpack(input_file, target_dir)?;
        }
        Commands::ListArchive {
            input_file,
        } => {
            kwui_cli::packager::list(input_file)?;
        }
        Commands::BinaryRelease { source_dir } => {
            kwui_cli::binary_release::build_and_package(&source_dir)?;
        }
        Commands::TemplateRelease { source_dir, key } => {
            let key = key.unwrap_or_else(|| git_half_hash().unwrap_or(String::from("unknown")));
            kwui_cli::template_release::package(&source_dir, &key)?;
        }
        Commands::New { project_name } => {
            let output_dir = std::env::current_dir()?.join(&project_name);
            kwui_cli::new::new_project(&output_dir, "app", &project_name)?;
        }
    }
    Ok(())
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
            let dst = if dst.ends_with('/') {
                dst
            } else {
                dst + "/"
            };
            println!("scan_dir, add dir [{}]:[{}]", full_path, dst);
            dir_items.push(dst);
        } else if meta.is_file() {
            println!("scan_dir, add file [{}]:[{}]", full_path, dst);
            file_items.push(kwui_cli::PackItem {
                src: full_path.to_string(),
                dst,
            });
        }
    }
    Ok((file_items, dir_items))
}
