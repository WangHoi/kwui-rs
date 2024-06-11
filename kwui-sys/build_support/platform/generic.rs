use super::prelude::*;

pub struct Generic;

impl PlatformDetails for Generic {
    fn uses_freetype(&self, _config: &BuildConfiguration) -> bool {
        true
    }

    fn cmake_args(&self, config: &BuildConfiguration, builder: &mut CMakeArgsBuilder) {
        gn_args(config, builder)
    }

    fn link_libraries(&self, features: &Features) -> Vec<String> {
        link_libraries(features)
    }
}

pub fn gn_args(config: &BuildConfiguration, builder: &mut CMakeArgsBuilder) {
    builder.target_os_and_default_cpu(&config.target.system);
}

pub fn link_libraries(_features: &Features) -> Vec<String> {
    Vec::new()
    /*
    if features.gl {
        vec!["GL".into()]
    } else {
        Vec::new()
    }
    */
}
