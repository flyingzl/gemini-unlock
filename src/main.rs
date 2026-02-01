mod cli;
mod error;
mod patcher;
mod platform;

use anyhow::{Context, Result};
use clap::Parser;
use log::{error, info, warn};
use std::fs;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::AppError;
use crate::patcher::apply_patches;
use crate::platform::{chrome_state_path, current_os, is_chrome_running, stop_chrome};

fn main() {
    // Initialize logging system
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    if let Err(err) = run() {
        error!("Program execution failed: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    info!("Chrome Gemini patch tool started");
    info!("Parameters: kill_chrome={}, restore={}", cli.kill_chrome, cli.restore);

    let os = current_os()?;
    info!("Detected OS: {:?}", os);

    let chrome_state = chrome_state_path(os)?;
    info!("Chrome config path: {}", chrome_state.display());

    let backup_path = create_backup_path(&chrome_state)?;
    info!("Backup path: {}", backup_path.display());

    // Ensure Chrome is not running
    ensure_chrome_not_running(os, cli.kill_chrome)?;

    // Execute restore or apply patches
    if cli.restore {
        restore_from_backup(&backup_path, &chrome_state)?;
    } else {
        apply_patches_workflow(&chrome_state, &backup_path)?;
    }

    Ok(())
}

/// Create backup file path
fn create_backup_path(chrome_state: &PathBuf) -> Result<PathBuf> {
    chrome_state
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| chrome_state.with_file_name(format!("{name}.bak")))
        .ok_or_else(|| AppError::InvalidPath(chrome_state.clone()).into())
}

/// Ensure Chrome is not running
fn ensure_chrome_not_running(os: platform::OsKind, kill_chrome: bool) -> Result<()> {
    if is_chrome_running(os)? {
        if kill_chrome {
            info!("Chrome is running, attempting to close...");
            stop_chrome(os)?;
            if is_chrome_running(os)? {
                error!("Chrome is still running, cannot continue");
                return Err(AppError::ChromeStillRunning.into());
            }
            info!("Chrome closed successfully");
        } else {
            error!("Chrome is running, please close it first or use --kill-chrome flag");
            return Err(AppError::ChromeRunning.into());
        }
    } else {
        info!("Chrome is not running");
    }
    Ok(())
}

/// Restore configuration from backup
fn restore_from_backup(backup_path: &PathBuf, chrome_state: &PathBuf) -> Result<()> {
    if !backup_path.exists() {
        error!("Backup file not found: {}", backup_path.display());
        return Err(AppError::BackupNotFound(backup_path.clone()).into());
    }
    info!("Restoring from backup...");
    fs::copy(backup_path, chrome_state)
        .with_context(|| format!("Restore failed: {}", chrome_state.display()))?;
    println!("✅ Restored from backup, please restart Chrome");
    info!("Restore completed");
    Ok(())
}

/// Apply patches workflow
fn apply_patches_workflow(chrome_state: &PathBuf, backup_path: &PathBuf) -> Result<()> {
    // Check if config file exists
    if !chrome_state.exists() {
        error!("Chrome config file not found: {}", chrome_state.display());
        return Err(AppError::ConfigNotFound(chrome_state.clone()).into());
    }

    // Create backup
    info!("Creating backup to: {}", backup_path.display());
    fs::copy(chrome_state, backup_path)
        .with_context(|| format!("Backup failed: {}", backup_path.display()))?;
    info!("Backup completed");

    // Read, modify and write configuration
    info!("Reading config file...");
    let content = fs::read_to_string(chrome_state)
        .with_context(|| format!("Read failed: {}", chrome_state.display()))?;
    info!("Config file size: {} bytes", content.len());

    info!("Applying patches...");
    let report = apply_patches(&content)?;

    // Display results before writing
    print_patch_report(&report);

    info!("Writing config file...");
    fs::write(chrome_state, report.content)
        .with_context(|| format!("Write failed: {}", chrome_state.display()))?;
    info!("Write completed");

    Ok(())
}

/// Print patch application results
fn print_patch_report(report: &crate::patcher::PatchReport) {
    println!();
    if report.changed_is_glic {
        println!("✓ Enabled is_glic_eligible");
        info!("Modified is_glic_eligible = true");
    } else {
        println!("⚠️ is_glic_eligible field not found");
        warn!("is_glic_eligible field not found");
    }
    if report.changed_variations_country {
        println!("✓ Set variations_country = us");
        info!("Modified variations_country = us");
    } else {
        println!("⚠️ variations_country field not found");
        warn!("variations_country field not found");
    }
    if report.changed_variations_permanent_country {
        println!("✓ Set variations_permanent_consistency_country = us");
        info!("Modified variations_permanent_consistency_country = [\"us\"]");
    } else {
        println!("⚠️ variations_permanent_consistency_country field not found");
        warn!("variations_permanent_consistency_country field not found");
    }

    println!();
    println!("✅ Done, please restart Chrome");
    info!("All operations completed");
}
