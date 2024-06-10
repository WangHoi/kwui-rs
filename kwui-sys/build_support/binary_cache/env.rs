use crate::build_support::cargo;

/// Returns `true` if the download of prebuilt binaries should be forced.
///
/// This can be used to test and download prebuilt binaries from within a repository build.
/// If this environment variable is not set, binaries are downloaded from crate builds only.
pub fn force_kwui_binaries_download() -> bool {
    cargo::env_var("FORCE_KWUI_BINARIES_DOWNLOAD").is_some()
}

/// The URL template to download the kwui binaries from.
///
/// `{tag}` will be replaced by the Tag (usually the released kwui-binding's crate's version).
/// `{key}` will be replaced by the Key (a combination of the repository hash, target, and features).
///
/// `file://` URLs are supported for local testing.
pub fn kwui_binaries_url() -> Option<String> {
    cargo::env_var("KWUI_BINARIES_URL")
}

/// The default URL template to download the binaries from.
pub fn kwui_binaries_url_default() -> String {
    "https://github.com/wanghoi/kwui-binaries/releases/download/{tag}/kwui-binaries-{key}.tar.gz"
        .into()
}

/// Force to build kwui, even if there is a binary available.
pub fn force_kwui_build() -> bool {
    cargo::env_var("FORCE_KWUI_BUILD").is_some()
}
