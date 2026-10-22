// Copyright (C) 2026 EfeDursun125
// This file is part of AMDGPU Tuner; see NOTICE and LICENSE.
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
