// Copyright (C) 2026 EfeDursun125
// This file is part of AMDGPU Tuner; see NOTICE and LICENSE.
// SPDX-License-Identifier: GPL-3.0-only
mod engine;
mod hardware;
mod theme;
use engine::{Profile, SETTINGS, System};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::{cell::RefCell, process::Command, rc::Rc};
slint::include_modules!();
const COPYRIGHT: &str = "Copyright (C) 2026 EfeDursun125";

struct State {
    system: System,
    profile: Profile,
    saved: Profile,
}
fn update(ui: &AppWindow, state: &State) {
    let rows: Vec<SettingRow> = SETTINGS
        .iter()
        .map(|setting| {
            let value = state.system.read(setting, &state.profile);
            SettingRow {
                title: setting.title.into(),
                variable: setting.variable.into(),
                description: setting.description.into(),
                state: value.label.into(),
                source: value.source.into(),
                checked: value.checked.unwrap_or(false),
                known: value.checked.is_some(),
                manual: state.profile.overrides.contains_key(setting.id),
                group: setting.group,
                available: state.system.setting_available(setting),
            }
        })
        .collect();
    // Keep repeater delegates alive so Material switches retain their animation state.
    let model = ui.get_settings();
    if let Some(existing) = model.as_any().downcast_ref::<VecModel<SettingRow>>()
        && existing.row_count() == rows.len()
    {
        for (index, row) in rows.into_iter().enumerate() {
            if existing.row_data(index).as_ref() != Some(&row) {
                existing.set_row_data(index, row);
            }
        }
    } else {
        ui.set_settings(ModelRc::new(VecModel::from(rows)));
    }
    ui.set_manual_count(
        SETTINGS
            .iter()
            .filter(|setting| {
                state.system.setting_available(setting)
                    && state.profile.overrides.contains_key(setting.id)
            })
            .count() as i32,
    );
    ui.set_dirty(state.profile != state.saved);
    ui.set_steam_line(state.system.steam_line(&state.profile).into());
    ui.set_summary(
        "System values are inherited. Manual overrides are exported only when applied.".into(),
    );
    let mut report = format!(
        "PROFILE\n{}\n\nSESSION FILE\n{} · {}\n\nINHERITED ENVIRONMENT\n",
        engine::profile_path().display(),
        engine::session_path().display(),
        if engine::session_path().exists() {
            "present"
        } else {
            "absent"
        }
    );
    let mut keys: Vec<_> = SETTINGS.iter().map(|s| s.variable).collect();
    keys.extend([
        "DISABLE_LAYER_MESA_ANTI_LAG",
        "DISABLE_LOW_LATENCY_LAYER",
        "RADV_FORCE_VRS_CONFIG_FILE",
    ]);
    keys.sort();
    keys.dedup();
    for key in keys {
        report.push_str(&format!(
            "{key} = {}\n",
            state
                .system
                .environment
                .get(key)
                .map(String::as_str)
                .unwrap_or("unset")
        ));
    }
    if !state.system.setting_available(&SETTINGS[1]) {
        report.push_str("\nLow Latency Layer is not installed. Its setting is hidden and saved overrides are inactive.\n");
    }
    report.push_str("\nLAYER MANIFESTS\n");
    for layer in &state.system.layers {
        if layer.name.contains("anti_lag") || layer.name.contains("low_latency") {
            report.push_str(&format!(
                "{}\n{}\nLibrary present: {}\nEnable: {:?}\nDisable: {:?}\n\n",
                layer.name, layer.path, layer.library_present, layer.enable, layer.disable
            ));
        }
    }
    report.push_str("SCOPE\nThese values describe the environment inherited by applications launched here. Other games or Steam may have a different environment. Refresh cannot change this process's inherited environment.\n\nTurning off RADV_PERFTEST / RADV_DEBUG / RADV_EXPERIMENTAL flags only removes the selected flag; it does not force off a driver default. Unknown flags are preserved.\n\nSOURCES\nhttps://docs.mesa3d.org/envvars.html\nhttps://material.slint.dev/getting-started/\n");
    report.push_str(&format!("\nLICENSE\nAMDGPU Tuner · {COPYRIGHT}\nGNU GPL version 3 only · Without any warranty\nFull terms: amdgpu-tuner --license\n"));
    for note in &state.system.notes {
        report.push_str(&format!("\n{note}\n"));
    }
    ui.set_environment_report(report.trim_end().into());
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|s| s == "--license") {
        print!(
            "{}\n{}",
            include_str!("../NOTICE"),
            include_str!("../LICENSE")
        );
        return Ok(());
    }
    if args.first().is_some_and(|s| s == "--help") {
        println!(
            "AMDGPU Tuner\n{COPYRIGHT}\nGPL-3.0-only; without any warranty. Use --license for full terms.\n\n  amdgpu-tuner                    Open the Material 3 interface\n  amdgpu-tuner --license          Show copyright, license and warranty terms\n  amdgpu-tuner --inspect          Show environment and profile values\n  amdgpu-tuner --steam-line       Print Steam launch options\n  amdgpu-tuner --run PROGRAM ...  Launch with the saved profile\n  amdgpu-tuner --write-session    Write the saved profile to environment.d\n  amdgpu-tuner --remove-session   Remove the application session file\n  amdgpu-tuner --screenshot PATH [0|1|2]  Capture a page as PNG"
        );
        return Ok(());
    }
    if args.first().is_some_and(|s| s == "--remove-session") {
        System::default().write_session(&Profile::default(), &engine::session_path())?;
        println!(
            "Session file removed. Log in again to clear values already inherited by processes."
        );
        return Ok(());
    }
    let system = System::detect();
    let profile = Profile::load(&engine::profile_path())?;
    match args.first().map(String::as_str) {
        Some("--inspect") => {
            for setting in SETTINGS {
                if !system.setting_available(setting) {
                    continue;
                }
                let r = system.read(setting, &profile);
                println!("{}: {}\n  {}", setting.variable, r.label, r.source);
            }
            return Ok(());
        }
        Some("--steam-line") => {
            println!("{}", system.steam_line(&profile));
            return Ok(());
        }
        Some("--run") => {
            let status = system.launch(&profile, &args[1..])?.wait()?;
            std::process::exit(status.code().unwrap_or(1));
        }
        Some("--write-session") => {
            system.write_session(&profile, &engine::session_path())?;
            println!(
                "Written for the next user session: {}",
                engine::session_path().display()
            );
            return Ok(());
        }
        Some("--screenshot") if (2..=5).contains(&args.len()) => (),
        Some("--ui-smoke-test") => (),
        Some(_) => return Err("Unknown or missing argument. Use --help.".into()),
        None => (),
    }
    let state = Rc::new(RefCell::new(State {
        system,
        saved: profile.clone(),
        profile,
    }));
    let ui = AppWindow::new()?;
    ui.on_matches(|title, variable, query| {
        let q = query.to_lowercase();
        title.to_lowercase().contains(&q) || variable.to_lowercase().contains(&q)
    });
    ui.set_hardware(hardware::detect().into());
    ui.set_status(
        "Ready. Save a profile, launch an application, or apply overrides to a future session."
            .into(),
    );
    update(&ui, &state.borrow());
    theme::apply(&ui);
    if args.first().is_some_and(|s| s == "--screenshot") && args.len() >= 3 {
        let page = args[2].parse::<i32>()?;
        if !(0..=2).contains(&page) {
            return Err("Page must be 0, 1 or 2".into());
        }
        ui.set_page(page);
        if args.len() >= 4 {
            let (width, height) = args[3].split_once('x').ok_or("Expected WIDTHxHEIGHT")?;
            let width = width.parse::<f32>()?;
            let height = height.parse::<f32>()?;
            let scale = std::env::var("SLINT_SCALE_FACTOR")
                .ok()
                .and_then(|value| value.parse::<f32>().ok())
                .unwrap_or(ui.window().scale_factor());
            ui.window().set_size(slint::PhysicalSize::new(
                (width * scale).round() as u32,
                (height * scale).round() as u32,
            ));
        }
        let weak = ui.as_weak();
        let hover = args.get(4).cloned().unwrap_or_default();
        slint::Timer::single_shot(std::time::Duration::from_millis(1200), move || {
            if let Some(ui) = weak.upgrade() {
                let scale = ui.window().scale_factor();
                let size = ui.window().size();
                let width = size.width as f32 / scale;
                let height = size.height as f32 / scale;
                let position = match hover.as_str() {
                    "hover-refresh" => slint::LogicalPosition::new(
                        width - 24. - ui.get_refresh_control_width() / 2.,
                        56.,
                    ),
                    "hover-save" => slint::LogicalPosition::new(
                        width - 24. - ui.get_save_control_width() / 2.,
                        height - 96.,
                    ),
                    _ => slint::LogicalPosition::new(-10., -10.),
                };
                ui.window()
                    .dispatch_event(slint::platform::WindowEvent::PointerMoved { position });
            }
        });
    }
    macro_rules! handler { ($method:ident, |$window:ident, $state:ident $(, $arg:ident)*| $body:block) => {{
        let weak = ui.as_weak(); let state = state.clone();
        ui.$method(move |$($arg),*| { if let Some($window) = weak.upgrade() { let mut guard = state.borrow_mut(); let $state = &mut *guard; $body update(&$window, $state); } });
    }}; }
    handler!(on_toggle_setting, |window, state, index, checked| {
        if let Some(setting) = SETTINGS
            .get(index as usize)
            .filter(|setting| state.system.setting_available(setting))
        {
            state.profile.overrides.insert(setting.id.into(), checked);
        }
        window.set_status(
            "Manual override updated. Driver behavior depends on application and driver support."
                .into(),
        );
    });
    handler!(on_inherit_setting, |window, state, index| {
        if let Some(setting) = SETTINGS.get(index as usize) {
            state.profile.overrides.remove(setting.id);
        }
        window.set_status("This setting now inherits its system value.".into());
    });
    handler!(on_reset_all, |window, state| {
        state.profile = Profile::default();
        window.set_status("Manual overrides cleared. Remove the session file from Launch & apply to undo session overrides too.".into());
    });
    handler!(on_refresh, |window, state| {
        state.system = System::detect();
        window.set_hardware(hardware::detect().into());
        theme::apply(&window);
        window.set_status("GPU list, layer manifests and theme refreshed. The environment remains the one inherited at startup.".into());
    });
    handler!(on_save_profile, |window, state| {
        match state.profile.save(&engine::profile_path()) {
            Ok(()) => {
                state.saved = state.profile.clone();
                window.set_status(
                    "Profile saved. Use --run or the Launch & apply page to use it.".into(),
                );
            }
            Err(e) => window.set_status(format!("Save failed: {e}").into()),
        }
    });
    handler!(on_apply_session, |window, state| {
        match state.system.write_session(&state.profile, &engine::session_path()) { Ok(()) => window.set_status("Session file updated for the next systemd user session. Running applications are unchanged.".into()), Err(e) => window.set_status(format!("Apply failed: {e}").into()) }
    });
    handler!(on_remove_session, |window, state| {
        match state
            .system
            .write_session(&Profile::default(), &engine::session_path())
        {
            Ok(()) => window.set_status(
                "Session file removed. Log in again to clear values already inherited by processes."
                    .into(),
            ),
            Err(e) => window.set_status(e.into()),
        }
    });
    handler!(on_copy_steam, |window, state| {
        let text = state.system.steam_line(&state.profile);
        let command = if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            "wl-copy"
        } else {
            "xclip"
        };
        let result = (|| -> std::io::Result<()> {
            use std::io::Write;
            let mut cmd = Command::new(command);
            if command == "xclip" {
                cmd.args(["-selection", "clipboard"]);
            }
            let mut child = cmd.stdin(std::process::Stdio::piped()).spawn()?;
            child.stdin.take().unwrap().write_all(text.as_bytes())?;
            if !child.wait()?.success() {
                return Err(std::io::Error::other("Clipboard command failed"));
            }
            Ok(())
        })();
        window.set_status(match result {
            Ok(()) => "Steam launch options copied to the clipboard.".into(),
            Err(e) => {
                format!("Clipboard: {e}. You can select and copy the text from the field.").into()
            }
        });
    });
    handler!(on_launch_command, |window, state, command| {
        let result = shell_words::split(&command)
            .map_err(|e| e.to_string())
            .and_then(|argv| state.system.launch(&state.profile, &argv));
        match result {
            Ok(mut child) => {
                let pid = child.id();
                std::thread::spawn(move || {
                    let _ = child.wait();
                });
                window.set_status(format!("Launched with profile overrides · PID {pid}").into());
            }
            Err(e) => window.set_status(format!("Launch failed: {e}").into()),
        }
    });
    let timer = slint::Timer::default();
    let weak = ui.as_weak();
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_secs(2),
        move || {
            if let Some(window) = weak.upgrade() {
                theme::apply(&window);
            }
        },
    );
    if args.first().is_some_and(|s| s == "--screenshot") {
        let path = args[1].clone();
        let weak = ui.as_weak();
        slint::Timer::single_shot(std::time::Duration::from_secs(3), move || {
            if let Some(ui) = weak.upgrade() {
                let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                    let pixels = ui.window().take_snapshot()?;
                    let file = std::fs::File::create(&path)?;
                    let mut encoder = png::Encoder::new(file, pixels.width(), pixels.height());
                    encoder.set_color(png::ColorType::Rgba);
                    encoder.set_depth(png::BitDepth::Eight);
                    encoder
                        .write_header()?
                        .write_image_data(pixels.as_bytes())?;
                    Ok(())
                })();
                if let Err(e) = result {
                    eprintln!("Screenshot failed: {e}");
                    std::process::exit(1);
                }
                let _ = ui.hide();
                let _ = slint::quit_event_loop();
            }
        });
    }
    if args.first().is_some_and(|s| s == "--ui-smoke-test") {
        // Keep pointer/animation fixtures independent of the host's GPU count.
        ui.set_hardware("AMD Graphics · Mesa / RADV".into());
        ui.window().set_size(slint::LogicalSize::new(1120., 900.));
        let weak = ui.as_weak();
        let state = state.clone();
        slint::Timer::single_shot(std::time::Duration::from_secs(1), move || {
            use slint::platform::{PointerEventButton, WindowEvent};
            let ui = weak.unwrap();
            assert!(
                state.borrow().profile.overrides.is_empty(),
                "UI test needs an empty isolated profile"
            );
            let click = |x, y| {
                let position = slint::LogicalPosition::new(x, y);
                ui.window().dispatch_event(WindowEvent::PointerPressed {
                    position,
                    button: PointerEventButton::Left,
                });
                ui.window().dispatch_event(WindowEvent::PointerReleased {
                    position,
                    button: PointerEventButton::Left,
                });
            };
            // Click the actual first Material switch, then toggle it back off.
            let original_model = ui.get_settings();
            click(1040., 376.);
            assert!(
                ui.get_settings() == original_model,
                "Switch delegate model was replaced"
            );
            assert_eq!(
                state.borrow().profile.overrides.get("anti-lag"),
                Some(&true)
            );
            click(1040., 376.);
            assert_eq!(
                state.borrow().profile.overrides.get("anti-lag"),
                Some(&false)
            );
            // The per-setting restore button must remove the override entirely.
            click(970., 376.);
            assert!(state.borrow().profile.overrides.is_empty());
            // An unset driver default must start unknown and remain manually controllable.
            ui.set_query("vblank_mode".into());
            let _ = ui.window().take_snapshot().unwrap();
            let vblank_y = 344.;
            click(1040., vblank_y);
            assert_eq!(state.borrow().profile.overrides.get("vsync"), Some(&true));
            click(1040., vblank_y);
            assert_eq!(state.borrow().profile.overrides.get("vsync"), Some(&false));
            click(970., vblank_y);
            assert!(state.borrow().profile.overrides.is_empty());
            ui.set_query("".into());
            let _ = ui.window().take_snapshot().unwrap();
            ui.invoke_toggle_setting(3, true);
            assert_eq!(ui.get_manual_count(), 1);
            ui.invoke_reset_all();
            assert_eq!(ui.get_manual_count(), 0);
            assert!(!ui.get_dirty());
            println!(
                "UI smoke: actual Material switch, unknown default on/off, restore and reset: PASS"
            );
            let weak = ui.as_weak();
            slint::Timer::single_shot(std::time::Duration::from_millis(300), move || {
                check_switch_animation(&weak.unwrap(), true);
            });
        });
    }
    ui.run()?;
    Ok(())
}

// Capture only the switch track: labels and other status changes cannot satisfy this check.
fn switch_pixels(ui: &AppWindow) -> Vec<u8> {
    let pixels = ui.window().take_snapshot().unwrap();
    let scale = ui.window().scale_factor() as f64;
    let mut result = Vec::new();
    for y in (360. * scale) as usize..(392. * scale) as usize {
        let start = (y * pixels.width() as usize + (1013. * scale) as usize) * 4;
        let end = (y * pixels.width() as usize + (1067. * scale) as usize) * 4;
        result.extend_from_slice(&pixels.as_bytes()[start..end]);
    }
    result
}
fn check_switch_animation(ui: &AppWindow, on: bool) {
    use slint::platform::{PointerEventButton, WindowEvent};
    let position = slint::LogicalPosition::new(1040., 376.);
    ui.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    ui.window().dispatch_event(WindowEvent::PointerReleased {
        position,
        button: PointerEventButton::Left,
    });
    assert_eq!(ui.get_settings().row_data(0).unwrap().checked, on);
    let start = switch_pixels(ui);
    let weak = ui.as_weak();
    slint::Timer::single_shot(std::time::Duration::from_millis(80), move || {
        let ui = weak.unwrap();
        let middle = switch_pixels(&ui);
        assert!(
            start != middle,
            "Switch did not animate between initial and intermediate frames"
        );
        let weak = ui.as_weak();
        slint::Timer::single_shot(std::time::Duration::from_millis(400), move || {
            let ui = weak.unwrap();
            let end = switch_pixels(&ui);
            assert!(
                middle != end,
                "Switch skipped the intermediate animation frame"
            );
            println!(
                "UI animation {on}: initial, intermediate and final switch frames differ: PASS"
            );
            if on {
                let weak = ui.as_weak();
                slint::Timer::single_shot(std::time::Duration::from_millis(100), move || {
                    check_switch_animation(&weak.unwrap(), false);
                });
            } else {
                let _ = ui.hide();
                let _ = slint::quit_event_loop();
            }
        });
    });
}
