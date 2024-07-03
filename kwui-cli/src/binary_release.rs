use std::fs::File;
use std::path::PathBuf;
use anyhow;
use flate2::Compression;
use flate2::write::GzEncoder;
use tar::Builder;
use std::io::{BufRead, BufReader};
use cargo_toml::Manifest;
use path_absolutize::*;
use crate::check_source_dir;

const TARGETS: &[&'static str] = &[
    "x86_64-pc-windows-msvc",
    "aarch64-linux-android",
];

pub fn build_and_package(source_dir: &PathBuf) -> anyhow::Result<()> {
    for tgt in TARGETS.iter() {
        build_and_package_target(source_dir, *tgt, false)?;
        if tgt.ends_with("-msvc") {
            build_and_package_target(source_dir, *tgt, true)?;
        }
    }
    Ok(())
}

fn build_and_package_target(source_dir: &PathBuf, target: &str, static_crt: bool) -> anyhow::Result<()> {
    println!("Checking source_dir [{}] ...", source_dir.display());
    check_source_dir(source_dir)?;

    let staging_dir = prepare_staging_dir(target, static_crt)?;
    println!("Staging dir: {}", staging_dir.display());

    println!("Building on source_dir [{}], for target '{}'{}' ...", source_dir.display(), target,
             if static_crt {
                 ", with static CRT"
             } else {
                 ""
             });
    let status = {
        let mut cmd = std::process::Command::new("cmd");
        cmd.current_dir(source_dir)
           .env("BUILD_ARTIFACTSTAGINGDIRECTORY", &staging_dir);
        if static_crt {
            cmd.env("RUSTFLAGS", "-Ctarget-feature=+crt-static");
        };
        cmd
            .args(["/c", "cargo", "build", "--release", "--target", target, "-p", "kwui-sys", "-vv"])
            .status()?
    };
    if !status.success() {
        anyhow::bail!("BUILD TARGET {} failed: {}", target, status);
    }

    println!("Packaging dir: {}", staging_dir.display());
    package(&staging_dir)?;

    Ok(())
}

fn prepare_staging_dir(target: &str, static_crt: bool) -> anyhow::Result<PathBuf> {
    let staging_dir = if static_crt {
        PathBuf::from("build").join(format!("{}-static", target))
    } else {
        PathBuf::from("build").join(target)
    };
    std::fs::create_dir_all(&staging_dir)?;
    Ok(staging_dir.absolutize().unwrap().into())
}

const KWUI_BINARIES: &'static str = "kwui-binaries";

fn package(staging_dir: &PathBuf) -> anyhow::Result<()> {
    let (tag, key) = parse_tag_key(staging_dir)?;
    let out_filename = format!("{}-{}.tar.gz", KWUI_BINARIES, key);
    let tar_gz = File::create(&out_filename)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);
    tar.append_dir_all(".", staging_dir.join(KWUI_BINARIES))?;
    tar.finish()?;

    println!("Done, output file [{}]", out_filename);

    Ok(())
}

fn parse_tag_key(staging_dir: &PathBuf) -> anyhow::Result<(String, String)> {
    let mut tag_file = BufReader::new(
        File::open(staging_dir.join(KWUI_BINARIES).join("tag.txt"))?);
    let mut tag = String::new();
    tag_file.read_line(&mut tag);
    drop(tag_file);

    let mut key_file = BufReader::new(
        File::open(staging_dir.join(KWUI_BINARIES).join("key.txt"))?);
    let mut key = String::new();
    key_file.read_line(&mut key);
    drop(key_file);

    Ok((tag, key))
}
