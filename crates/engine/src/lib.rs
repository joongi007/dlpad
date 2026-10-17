//! Blocking adapters must run outside the UI thread.
pub mod activation;
pub mod app_install;
pub mod app_update;
pub mod cookies;
mod deno_runtime;
pub mod download;
pub mod file_identity;
pub mod instance;
pub mod library;
pub mod location;
pub mod metrics;
pub mod process;
pub mod reports;
#[cfg(any(windows, test))]
pub mod runtime_environment;
pub mod store;
pub mod thread_cpu;
pub mod tools;
pub mod watch;

#[cfg(target_os = "linux")]
pub mod processing_endpoint;

#[cfg(windows)]
mod windows_private_security;

// Native cache authority is integrated before guardian capability handoff.
#[cfg(windows)]
#[allow(dead_code)]
mod windows_owned_deno_cache;

// Original identity attachment is validated before native guardian handoff.
pub mod media_preview;
#[cfg(any(windows, test))]
#[allow(dead_code)]
mod windows_deno_cache_binding;

mod created_relative_file;
pub mod file_export;
