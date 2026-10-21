// SPDX-License-Identifier: GPL-3.0-only
fn main() {
    let config = slint_build::CompilerConfiguration::new().with_library_paths(
        std::collections::HashMap::from([(
            "material".into(),
            std::path::PathBuf::from("vendor/material-1.1.0/material.slint"),
        )]),
    );
    slint_build::compile_with_config("ui/main.slint", config).unwrap();
}
