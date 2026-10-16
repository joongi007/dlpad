//! Native accessibility candidate using the existing Rust domain and engine.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

// Glow entrypoint is shared but unused by this CPU-only host.
#[allow(dead_code)]
mod accessible;
mod accessible_diagnostics;
mod accessible_history;
mod accessible_library;
mod accessible_metrics;
mod accessible_options;
mod accessible_plan;
mod accessible_preview;
mod accessible_report_fixture;
mod accessible_settings;
mod accessible_tool_preferences;
mod accessible_tools;
#[cfg(test)]
mod accesskit_value_tests;
#[allow(dead_code)]
mod app_icon;
#[allow(dead_code)]
mod catalog;
mod diagnostic_path_picker;
mod display_rate;
mod external_open_broker;
#[cfg(all(windows, feature = "first-paint-fixture"))]
mod first_paint_fixture;
#[cfg(all(windows, feature = "first-paint-fixture"))]
mod frame_phase_fixture;
#[cfg(test)]
mod ime_cancel_regressions;
#[cfg(all(windows, feature = "ime-compose-fixture"))]
mod ime_compose_fixture;
#[cfg(all(windows, feature = "ime-owner-fixture"))]
mod ime_owner_fixture;
mod library_open;
#[cfg(all(windows, feature = "library-scan-closewait-fixture"))]
mod library_scan_closewait_fixture;
mod list_search;
#[cfg(windows)]
mod local_export_ui;
#[cfg(windows)]
mod local_export_worker;
#[allow(dead_code)]
mod managed;
mod managed_candidates;
mod media_detail_open;
mod media_preview_worker;
mod media_timeline;
mod media_workspace;
mod path_picker;
#[cfg(all(windows, feature = "report-closewait-fixture"))]
mod report_closewait_fixture;
mod software_host;
mod tool_path_picker;
mod tool_update_receipts;
mod ui_components;
mod ui_theme;
mod ui_tokens;
#[cfg(all(windows, feature = "first-paint-fixture"))]
mod worker_phase_fixture;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dlpad_engine::process::guardian_entry()?;
    dlpad_engine::process::register_self_guardian()?;
    #[cfg(all(windows, feature = "first-paint-fixture"))]
    first_paint_fixture::initialize(std::env::args_os().skip(1).collect())?;
    #[cfg(all(windows, feature = "report-closewait-fixture"))]
    report_closewait_fixture::initialize(std::env::args_os().skip(1).collect())?;
    #[cfg(all(windows, feature = "library-scan-closewait-fixture"))]
    library_scan_closewait_fixture::initialize(std::env::args_os().skip(1).collect())?;
    #[cfg(all(windows, feature = "ime-owner-fixture"))]
    ime_owner_fixture::initialize(std::env::args_os().skip(1).collect())?;
    #[cfg(all(windows, feature = "ime-compose-fixture"))]
    ime_compose_fixture::initialize(std::env::args_os().skip(1).collect())?;
    let root = std::env::var_os("DLPAD_TEST_DATA")
        .map(std::path::PathBuf::from)
        .map(Ok)
        .unwrap_or_else(dlpad_engine::location::default_data_dir)?;
    let guard = match dlpad_engine::instance::InstanceGuard::acquire(&root) {
        Ok(guard) => guard,
        Err(dlpad_engine::instance::InstanceError::AlreadyRunning) => {
            dlpad_engine::activation::send_restore(&root)?;
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };
    software_host::run(guard.root().to_path_buf())?;
    drop(guard);
    Ok(())
}

mod tool_update_ui;
