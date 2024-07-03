use std::path::PathBuf;
use anyhow;
use std::process::{Command, Stdio};
use cargo_toml::Manifest;

pub fn build_windows(project_dir: &PathBuf, verbose: bool, release: bool) ->anyhow::Result<()> {
    let profile = if release {
        "release"
    } else {
        "debug"
    };
    println!("BUILDING windows {} executable ...", profile);
    let mut args = vec!["build"];
    if release {
        args.extend(["--release"]);
    }
    if verbose {
        args.extend(["-vv"]);
    }
    Command::new("cargo")
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    Ok(())
}

pub fn build_apk(project_dir: &PathBuf, verbose: bool, release: bool) ->anyhow::Result<()> {
    let profile = if release {
        "release"
    } else {
        "debug"
    };
    println!("BUILDING android {} apk ...", profile);
    let mut cargo_args = vec!["build", "--target", "aarch64-linux-android"];
    if release {
        cargo_args.extend(["--release"]);
    }
    if verbose {
        cargo_args.extend(["-vv"]);
    }
    Command::new("cargo")
        .current_dir(project_dir)
        .args(&cargo_args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    let lib_filename = format!("lib{}.so", crate::cargo_package_name(project_dir)?);

    println!("COPYING [{}]",
             project_dir.join("target/aarch64-linux-android").join(profile).join(&lib_filename).display());
    std::fs::copy(
        project_dir.join("target/aarch64-linux-android").join(profile).join(&lib_filename),
        project_dir.join("android/app/src/main/jniLibs/arm64-v8a").join(&lib_filename),
    )?;

    println!("GRADLE building...");
    let mut gradle_args = vec!["/c", "gradlew.bat"];
    if release {
        gradle_args.extend(["assembleRelease"]);
    } else {
        gradle_args.extend(["assembleDebug"]);
    }
    gradle_args.extend(["--no-daemon"]);
    Command::new("cmd")
        .current_dir(project_dir.join("android"))
        .args(&gradle_args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    let apk_output_dir = project_dir.join("android/app/build/outputs/apk").join(profile);
    let apk_output_name = if release {
        "app-release-unsigned.apk"
    } else {
        "app-debug.apk"
    };
    println!("BUILT apk [{}]", apk_output_dir.join(apk_output_name).display());

    Ok(())
}
