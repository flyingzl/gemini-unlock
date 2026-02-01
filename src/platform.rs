use std::collections::HashSet;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::thread;
use std::time::Duration;

use crate::error::{AppError, AppResult};

/// Supported operating system types.
///
/// # Examples
///
/// ```text
/// OsKind::Macos
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OsKind {
    /// macOS.
    Macos,
    /// Linux.
    Linux,
    /// Windows.
    Windows,
}

/// Get current operating system type.
///
/// # Examples
///
/// ```text
/// let os = current_os()?;
/// ```
pub fn current_os() -> AppResult<OsKind> {
    match std::env::consts::OS {
        "macos" => Ok(OsKind::Macos),
        "linux" => Ok(OsKind::Linux),
        "windows" => Ok(OsKind::Windows),
        other => Err(AppError::UnsupportedOs(other.to_string())),
    }
}

/// Get Chrome Local State path.
///
/// # Examples
///
/// ```text
/// let path = chrome_state_path(OsKind::Macos)?;
/// ```
pub fn chrome_state_path(os: OsKind) -> AppResult<PathBuf> {
    match os {
        OsKind::Macos => {
            let home =
                std::env::var("HOME").map_err(|_| AppError::MissingEnv("HOME".to_string()))?;
            Ok(PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Google")
                .join("Chrome")
                .join("Local State"))
        }
        OsKind::Windows => {
            let local_app_data = std::env::var("LOCALAPPDATA")
                .map_err(|_| AppError::MissingEnv("LOCALAPPDATA".to_string()))?;
            Ok(PathBuf::from(local_app_data)
                .join("Google")
                .join("Chrome")
                .join("User Data")
                .join("Local State"))
        }
        OsKind::Linux => {
            let home =
                std::env::var("HOME").map_err(|_| AppError::MissingEnv("HOME".to_string()))?;
            Ok(PathBuf::from(home)
                .join(".config")
                .join("google-chrome")
                .join("Local State"))
        }
    }
}

/// Check if Chrome is running.
///
/// # Examples
///
/// ```text
/// let running = is_chrome_running(OsKind::Macos)?;
/// ```
pub fn is_chrome_running(os: OsKind) -> AppResult<bool> {
    match os {
        OsKind::Macos => pgrep_running("Google Chrome"),
        OsKind::Linux => {
            if pgrep_running("chrome")? {
                Ok(true)
            } else {
                pgrep_running("google-chrome")
            }
        }
        OsKind::Windows => {
            let output = Command::new("tasklist")
                .args(["/FI", "IMAGENAME eq chrome.exe"])
                .output()?;
            if !output.status.success() {
                return Err(command_failed(
                    "tasklist /FI \"IMAGENAME eq chrome.exe\"",
                    &output,
                ));
            }
            let stdout = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
            Ok(stdout.contains("chrome.exe"))
        }
    }
}

/// Close running Chrome.
///
/// # Examples
///
/// ```text
/// stop_chrome(OsKind::Windows)?;
/// ```
pub fn stop_chrome(os: OsKind) -> AppResult<()> {
    match os {
        OsKind::Macos => {
            let output = Command::new("osascript")
                .args(["-e", "quit app \"Google Chrome\""])
                .output()?;
            if !output.status.success() {
                return Err(command_failed(
                    "osascript -e 'quit app \"Google Chrome\"'",
                    &output,
                ));
            }
        }
        OsKind::Windows => {
            let output = Command::new("taskkill")
                .args(["/IM", "chrome.exe", "/F"])
                .output()?;
            if !output.status.success() {
                return Err(command_failed("taskkill /IM chrome.exe /F", &output));
            }
        }
        OsKind::Linux => {
            let pids = pgrep_pids(&["chrome", "google-chrome"])?;
            if pids.is_empty() {
                return Ok(());
            }
            send_signal(&pids, "TERM")?;
        }
    }

    // Wait for process exit (using improved waiting mechanism)
    wait_for_process_stop(os, Duration::from_secs(3), Duration::from_millis(300))?;

    // Linux special handling: if TERM signal fails, try KILL
    if os == OsKind::Linux && is_chrome_running(os)? {
        log::warn!("TERM signal failed, trying KILL signal");
        let pids = pgrep_pids(&["chrome", "google-chrome"])?;
        if !pids.is_empty() {
            send_signal(&pids, "KILL")?;
        }
        wait_for_process_stop(os, Duration::from_secs(3), Duration::from_millis(300))?;
    }

    if is_chrome_running(os)? {
        Err(AppError::ChromeStillRunning)
    } else {
        Ok(())
    }
}

/// Wait for process to stop
///
/// # Arguments
///
/// * `os` - Operating system type
/// * `timeout` - Total timeout duration
/// * `check_interval` - Check interval
fn wait_for_process_stop(os: OsKind, timeout: Duration, check_interval: Duration) -> AppResult<()> {
    let start = std::time::Instant::now();
    let mut attempts = 0;

    while start.elapsed() < timeout {
        attempts += 1;
        if !is_chrome_running(os)? {
            log::debug!("Process stopped at check #{}", attempts);
            return Ok(());
        }
        thread::sleep(check_interval);
    }

    log::warn!("Wait for process stop timeout ({:?}), total attempts: {}", timeout, attempts);
    Err(AppError::ChromeStillRunning)
}

fn pgrep_running(name: &str) -> AppResult<bool> {
    let status = Command::new("pgrep").arg("-x").arg(name).status()?;
    match status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        Some(code) => Err(AppError::CommandFailed {
            command: format!("pgrep -x \"{name}\""),
            details: format!("退出码 {code}"),
        }),
        None => Err(AppError::CommandFailed {
            command: format!("pgrep -x \"{name}\""),
            details: "被信号终止".to_string(),
        }),
    }
}

fn pgrep_pids(names: &[&str]) -> AppResult<Vec<i32>> {
    let mut pids = HashSet::new();
    for name in names {
        let output = Command::new("pgrep").arg("-x").arg(name).output()?;
        match output.status.code() {
            Some(0) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for item in stdout.split_whitespace() {
                    let pid: i32 = item.parse().map_err(|_| AppError::CommandFailed {
                        command: format!("pgrep -x \"{name}\""),
                        details: format!("无法解析 pid: {item}"),
                    })?;
                    pids.insert(pid);  // O(1) 插入，自动去重
                }
            }
            Some(1) => {}
            Some(code) => {
                return Err(AppError::CommandFailed {
                    command: format!("pgrep -x \"{name}\""),
                    details: format!("退出码 {code}"),
                });
            }
            None => {
                return Err(AppError::CommandFailed {
                    command: format!("pgrep -x \"{name}\""),
                    details: "Terminated by signal".to_string(),
                });
            }
        }
    }
    // Convert to sorted Vec for later use
    let mut result: Vec<i32> = pids.into_iter().collect();
    result.sort_unstable();
    Ok(result)
}

fn send_signal(pids: &[i32], signal: &str) -> AppResult<()> {
    if pids.is_empty() {
        return Ok(());
    }
    let mut cmd = Command::new("kill");
    cmd.arg(format!("-{signal}"));
    for pid in pids {
        cmd.arg(pid.to_string());
    }
    let output = cmd.output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(command_failed(
            &format!(
                "kill -{signal} {}",
                pids.iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            &output,
        ))
    }
}

fn command_failed(command: &str, output: &Output) -> AppError {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let details = if stdout.is_empty() && stderr.is_empty() {
        format!("Exit code {:?}", output.status.code())
    } else {
        format!("stdout: {stdout}; stderr: {stderr}")
    };

    AppError::CommandFailed {
        command: command.to_string(),
        details,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_kind_equality() {
        assert_eq!(OsKind::Macos, OsKind::Macos);
        assert_ne!(OsKind::Macos, OsKind::Linux);
        assert_ne!(OsKind::Linux, OsKind::Windows);
    }

    #[test]
    fn test_current_os_valid() {
        let os = current_os();
        assert!(os.is_ok(), "Should detect current OS");
        let os_kind = os.unwrap();
        // Verify returned OS is supported
        assert!(matches!(
            os_kind,
            OsKind::Macos | OsKind::Linux | OsKind::Windows
        ));
    }

    #[test]
    fn test_chrome_state_path_formats() {
        // Test macOS path format
        let macos_path = chrome_state_path(OsKind::Macos);
        assert!(macos_path.is_ok());
        let path = macos_path.unwrap();
        assert!(path.to_str().unwrap().contains("Local State"));

        // Test Linux path format (requires HOME env var)
        unsafe {
            std::env::set_var("HOME", "/tmp/test_home");
        }
        let linux_path = chrome_state_path(OsKind::Linux);
        assert!(linux_path.is_ok());
        let path = linux_path.unwrap();
        assert!(path.to_str().unwrap().contains("Local State"));

        // Test Windows path format (requires LOCALAPPDATA env var)
        unsafe {
            std::env::set_var("LOCALAPPDATA", "C:\\Users\\Test\\AppData\\Local");
        }
        let windows_path = chrome_state_path(OsKind::Windows);
        assert!(windows_path.is_ok());
        let path = windows_path.unwrap();
        assert!(path.to_str().unwrap().contains("Local State"));
    }

    #[test]
    fn test_command_failed_error_formatting() {
        use std::process::Command;

        // Test command failure error formatting
        let output = Command::new("sh")
            .arg("-c")
            .arg("echo error output >&2; echo stdout; exit 1")
            .output()
            .expect("Should execute test command");

        let err = command_failed("test command", &output);
        assert!(matches!(err, AppError::CommandFailed { .. }));
        if let AppError::CommandFailed { command, details } = err {
            assert_eq!(command, "test command");
            // Verify error info contains stdout and stderr
            println!("Error details: {details}");
        }
    }

    #[test]
    fn test_os_kind_copy() {
        // Test OsKind implements Copy
        let os1 = OsKind::Macos;
        let os2 = os1;  // Should trigger Copy, not Move
        assert_eq!(os1, OsKind::Macos);
        assert_eq!(os2, OsKind::Macos);
    }
}
