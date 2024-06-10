use crate::build_support::{
    binaries_config::BinariesConfiguration,
    binary_cache::binaries,
};
use std::path::Path;

/// Publish the binaries to Azure.
pub fn publish(binaries_config: &BinariesConfiguration, staging_directory: &Path) {
    println!(
        "DETECTED AZURE, exporting binaries to {}",
        staging_directory.to_str().unwrap()
    );

    println!("EXPORTING BINARIES");
    // let source_files = &[(KWUI_LICENSE, "LICENSE_SKIA")];
    let source_files = &[];
    binaries::export(binaries_config, source_files, staging_directory)
        .expect("EXPORTING BINARIES FAILED")
}
