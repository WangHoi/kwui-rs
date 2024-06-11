//! All the configuration settings that can be resolved statically for a platform target. From the
//! environment, and current build configurations.

use self::prelude::quote;
use super::{
    cargo::{self, Target},
    clang,
    features::Features,
    kwui::BuildConfiguration,
};

pub mod android;
mod generic;
mod windows;

#[allow(unused)]
pub fn uses_freetype(config: &BuildConfiguration) -> bool {
    details(&config.target).uses_freetype(config)
}

pub fn gn_args(config: &BuildConfiguration, mut builder: CMakeArgsBuilder) -> Vec<(String, String)> {
    details(&config.target).cmake_args(config, &mut builder);
    builder.into_gn_args()
}

#[derive(Clone, Debug)]
pub struct BindgenAndCCArgs {
    pub args: Vec<String>,
    pub target_override: Option<String>,
}

pub fn _bindgen_and_cc_args(target: &Target, sysroot: Option<&str>) -> BindgenAndCCArgs {
    let mut builder = BindgenArgsBuilder::new(sysroot);
    details(target).bindgen_args(target, &mut builder);
    builder.into_bindgen_and_cc_args()
}

pub fn link_libraries(features: &Features, target: &Target) -> Vec<String> {
    details(target).link_libraries(features)
}

pub fn filter_features(
    target: &Target,
    use_system_libraries: bool,
    features: Features,
) -> Features {
    details(target).filter_platform_features(use_system_libraries, features)
}

#[allow(unused)]
pub trait PlatformDetails {
    /// We need this information relatively early on to help parameterizing GN.
    fn uses_freetype(&self, _config: &BuildConfiguration) -> bool;
    fn cmake_args(&self, config: &BuildConfiguration, builder: &mut CMakeArgsBuilder);
    fn bindgen_args(&self, _target: &Target, _builder: &mut BindgenArgsBuilder) {}
    fn link_libraries(&self, features: &Features) -> Vec<String>;
    fn filter_platform_features(
        &self,
        _use_system_libraries: bool,
        features: Features,
    ) -> Features {
        features
    }
}

fn details(target: &Target) -> &dyn PlatformDetails {
    let host = cargo::host();
    match target.as_strs() {
        // ("wasm32", "unknown", "emscripten", _) => &emscripten::Emscripten,
        (_, "linux", "android", _) | (_, "linux", "androideabi", _) => &android::Android,
        // (_, "apple", "darwin", _) => &macos::MacOs,
        // (_, "apple", "ios", _) => &ios::Ios,
        (_, _, "windows", Some("msvc")) if host.is_windows() => &windows::Msvc,
        (_, _, "windows", _) => &windows::Generic,
        // (_, "unknown", "linux", Some("musl")) => &alpine::Musl,
        // (_, _, "linux", _) => &linux::Linux,
        _ => &generic::Generic,
    }
}

#[derive(Debug)]
pub struct CMakeArgsBuilder {
    target_arch: String,
    target_str: Option<String>,
    cmake_args: Vec<(String, String)>,
    cmake_cflags: Vec<String>,
}

impl CMakeArgsBuilder {
    pub fn new(target: &Target) -> Self {
        Self {
            target_arch: target.architecture.clone(),
            target_str: Some(target.to_string()),
            cmake_args: Vec::default(),
            cmake_cflags: Vec::default(),
        }
    }

    /// Overwrite the default target.
    pub fn _target(&mut self, target: impl Into<Option<String>>) {
        self.target_str = target.into();
    }

    /// Set a Skia GN arg.
    pub fn arg(&mut self, name: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.cmake_args.push((name.into(), value.into()));
        self
    }

    /// Set a Skia C flag.
    pub fn cflag(&mut self, flag: impl Into<String>) -> &mut Self {
        self.cmake_cflags.push(flag.into());
        self
    }

    /// Set multiple Skia C flags.
    pub fn cflags(&mut self, flags: impl IntoIterator<Item = String>) {
        flags.into_iter().for_each(|s| {
            self.cflag(s);
        });
    }

    /// Explicitly set `target_os` to the value and `target_cpu` to clang's default. By default,
    /// none of them are set.
    pub fn target_os_and_default_cpu(&mut self, os: impl Into<String>) {
        self.arg("target_os", quote(&os.into()));
        self.arg("target_cpu", quote(clang::target_arch(&self.target_arch)));
    }

    pub fn into_gn_args(mut self) -> Vec<(String, String)> {
        let mut asmflags = Vec::new();

        if let Some(target) = &self.target_str {
            let target = format!("--target={target}");
            self.cflag(&target);
            asmflags.push(target);
        }

        if !self.cmake_cflags.is_empty() {
            let cflags = format!(
                "[{}]",
                self.cmake_cflags
                    .iter()
                    .map(|s| quote(s))
                    .collect::<Vec<_>>()
                    .join(",")
            );

            self.arg("extra_cflags", cflags.clone());
        }

        if !asmflags.is_empty() {
            let asmflags = format!(
                "[{}]",
                asmflags
                    .iter()
                    .map(|s| quote(s))
                    .collect::<Vec<_>>()
                    .join(",")
            );
            self.arg("extra_asmflags", asmflags);
        }

        self.cmake_args.into_iter().collect()
    }
}

#[derive(Debug)]
pub struct BindgenArgsBuilder {
    /// sysroot if set explicitly.
    sysroot: Option<String>,
    sysroot_prefix: String,
    bindgen_and_cc_args: Vec<String>,
    target_override: Option<String>,
}

#[allow(unused)]
impl BindgenArgsBuilder {
    pub fn new(sysroot: Option<&str>) -> Self {
        Self {
            sysroot: sysroot.map(|s| s.into()),
            sysroot_prefix: "--sysroot=".into(),
            bindgen_and_cc_args: Vec::new(),
            target_override: None,
        }
    }

    pub fn sysroot(&self) -> Option<&str> {
        self.sysroot.as_deref()
    }

    /// Set the sysroot.
    pub fn set_sysroot(&mut self, sysroot: impl Into<String>) {
        self.sysroot = Some(sysroot.into())
    }

    /// If a sysroot is set, we use the default prefix `--sysroot=` for setting it, but some
    /// platforms may object.
    pub fn sysroot_prefix(&mut self, prefix: impl Into<String>) {
        self.sysroot_prefix = prefix.into();
    }

    /// Set a Bindgen Clang arg.
    pub fn arg(&mut self, arg: impl Into<String>) -> &mut Self {
        self.bindgen_and_cc_args.push(arg.into());
        self
    }

    /// Set multiple Bindgen Clang arguments.
    pub fn args(&mut self, arguments: impl IntoIterator<Item = String>) {
        arguments.into_iter().for_each(|s| {
            self.arg(s);
        });
    }

    // TODO: only return one Vec<>
    pub fn into_bindgen_and_cc_args(mut self) -> BindgenAndCCArgs {
        if let Some(sysroot) = &self.sysroot {
            let sysroot_arg = format!("{}{}", self.sysroot_prefix, sysroot);
            self.arg(&sysroot_arg);
        }

        BindgenAndCCArgs {
            args: self.bindgen_and_cc_args,
            target_override: self.target_override,
        }
    }

    pub fn override_target(&mut self, target: &str) {
        self.target_override = Some(target.to_owned());
    }
}

pub mod prelude {
    pub use self::{cargo::Target, kwui::BuildConfiguration};
    pub use super::{BindgenArgsBuilder, CMakeArgsBuilder, PlatformDetails};
    pub use crate::build_support::{cargo, clang, features::Features, kwui};

    pub fn quote(s: &str) -> String {
        format!("\"{s}\"")
    }

    pub fn yes() -> String {
        "true".into()
    }

    pub fn no() -> String {
        "false".into()
    }

    pub fn yes_if(y: bool) -> String {
        if y {
            yes()
        } else {
            no()
        }
    }
}
