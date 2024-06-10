/// Environment variables used for configuring the Skia build.
use crate::build_support::cargo;
use std::path::PathBuf;

/// A boolean specifying whether to build Skia's dependencies or not. If not, the system's
/// provided libraries are used.
pub fn use_system_libraries() -> bool {
    cargo::env_var("KWUI_USE_SYSTEM_LIBRARIES").is_some()
}

/// The full path of the ninja command to run.
pub fn ninja_command() -> Option<PathBuf> {
    cargo::env_var("KWUI_NINJA_COMMAND").map(PathBuf::from)
}

/// The full path of the cmake command to run.
pub fn cmake_command() -> Option<PathBuf> {
    cargo::env_var("KWUI_CMAKE_COMMAND").map(PathBuf::from)
}
