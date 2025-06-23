use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;

/// Check if a command exists in PATH
pub fn command_exists(command: &str) -> bool {
    which::which(command).is_ok()
}

/// Check if git is available
pub fn check_git_available() -> Result<()> {
    if !command_exists("git") {
        return Err(anyhow!("Git is not installed or not available in PATH"));
    }
    Ok(())
}

/// Check if cargo is available
pub fn check_cargo_available() -> Result<()> {
    if !command_exists("cargo") {
        return Err(anyhow!("Cargo is not installed or not available in PATH"));
    }
    Ok(())
}

/// Check if we're in a git repository
pub fn is_git_repository(path: &Path) -> bool {
    path.join(".git").exists()
}

/// Check if we're in a Rust project
pub fn is_rust_project(path: &Path) -> bool {
    path.join("Cargo.toml").exists()
}

/// Get the current git branch name
pub fn get_current_branch(repo_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to get current branch"));
    }

    let branch = String::from_utf8(output.stdout)?
        .trim()
        .to_string();

    if branch.is_empty() {
        return Err(anyhow!("No current branch found"));
    }

    Ok(branch)
}

/// Validate plugin name
pub fn validate_plugin_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("Plugin name cannot be empty"));
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(anyhow!("Plugin name can only contain alphanumeric characters, underscores, and hyphens"));
    }

    if name.starts_with('-') || name.starts_with('_') {
        return Err(anyhow!("Plugin name cannot start with a hyphen or underscore"));
    }

    Ok(())
}

/// Format bytes as human readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_plugin_name() {
        assert!(validate_plugin_name("my_plugin").is_ok());
        assert!(validate_plugin_name("my-plugin").is_ok());
        assert!(validate_plugin_name("plugin123").is_ok());
        assert!(validate_plugin_name("MyPlugin").is_ok());
        
        assert!(validate_plugin_name("").is_err());
        assert!(validate_plugin_name("_plugin").is_err());
        assert!(validate_plugin_name("-plugin").is_err());
        assert!(validate_plugin_name("my plugin").is_err());
        assert!(validate_plugin_name("my@plugin").is_err());
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1048576), "1.0 MB");
    }
}