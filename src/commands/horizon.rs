use clap::Subcommand;
use anyhow::{anyhow, Context, Result};
use colored::*;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use walkdir::WalkDir;
use toml_edit::{Document, value};

#[derive(Subcommand)]
pub enum HorizonCommand {
    /// Plugin related commands
    #[command(subcommand)]
    Plugin(PluginCommand),
}

#[derive(Subcommand)]
pub enum PluginCommand {
    /// Create a new Horizon plugin from template
    New {
        /// Name of the plugin
        name: String,
        /// Target directory (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Build a plugin (from plugin dir or Horizon repo root)
    Build {
        /// Plugin name (positional, required if in Horizon repo root)
        #[arg()]
        plugin: Option<String>,
        /// Horizon project path (defaults to ../Horizon)
        #[arg(long)]
        horizon_path: Option<PathBuf>,
        /// Skip copying to Horizon plugins directory
        #[arg(long)]
        no_copy: bool,
        /// Plugin name (optional, for --plugin usage)
        #[arg(long)]
        plugin_flag: Option<String>,
    },
}

pub async fn handle_command(cmd: HorizonCommand) -> Result<()> {
    match cmd {
        HorizonCommand::Plugin(plugin_cmd) => handle_plugin_command(plugin_cmd).await,
    }
}

async fn handle_plugin_command(cmd: PluginCommand) -> Result<()> {
    match cmd {
        PluginCommand::New { name, path } => create_new_plugin(&name, path).await,
        PluginCommand::Build { plugin, horizon_path, no_copy, plugin_flag } => {
            // Prefer positional plugin argument, fallback to --plugin
            let plugin_name = plugin.or(plugin_flag);
            build_plugin(horizon_path, no_copy, plugin_name).await
        }
    }
}

async fn create_new_plugin(name: &str, target_path: Option<PathBuf>) -> Result<()> {
    let target_dir = target_path.unwrap_or_else(|| PathBuf::from("."));
    let plugin_dir = target_dir.join(name);

    println!("🔧 Creating new Horizon plugin: {}", style(name).cyan().bold());
    println!("📂 Target directory: {}", style(plugin_dir.display()).yellow());

    // Create progress bar
    let pb = ProgressBar::new(4);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    // Step 1: Clone the sample repository
    pb.set_message("Cloning Horizon-Plugin-Sample...");
    clone_sample_repo(&plugin_dir).await?;
    pb.inc(1);

    // Step 2: Update Cargo.toml with new name
    pb.set_message("Updating Cargo.toml...");
    update_cargo_toml(&plugin_dir, name)?;
    pb.inc(1);

    // Step 3: Update plugin code
    pb.set_message("Updating plugin code...");
    update_plugin_code(&plugin_dir, name)?;
    pb.inc(1);

    // Step 4: Clean up
    pb.set_message("Cleaning up...");
    cleanup_plugin_directory(&plugin_dir)?;
    pb.inc(1);

    pb.finish_with_message("✅ Plugin created successfully!");
    
    println!();
    println!("{}", "🎉 Plugin created successfully!".green().bold());
    println!("📁 Plugin location: {}", style(plugin_dir.display()).yellow());
    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. cd {}", name);
    println!("  2. fbcli horizon plugin build");
    println!();

    Ok(())
}

async fn clone_sample_repo(target_dir: &Path) -> Result<()> {
    use git2::Repository;

    let repo_url = "https://github.com/Far-Beyond-Dev/Horizon-Plugin-Sample.git";
    
    // Clone the repository
    Repository::clone(repo_url, target_dir)
        .with_context(|| format!("Failed to clone sample repository to {}", target_dir.display()))?;
    
    Ok(())
}

fn update_cargo_toml(plugin_dir: &Path, plugin_name: &str) -> Result<()> {
    let cargo_path = plugin_dir.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_path)?;
    
    let mut doc = content.parse::<Document>()?;
    
    // Update package name
    doc["package"]["name"] = value(&format!("plugin_{}", plugin_name));
    
    fs::write(cargo_path, doc.to_string())?;
    Ok(())
}

fn update_plugin_code(plugin_dir: &Path, plugin_name: &str) -> Result<()> {
    let lib_path = plugin_dir.join("src/lib.rs");
    
    // Create a basic version of the greeter plugin with the new name
    let new_content = create_basic_plugin_template(plugin_name);
    
    fs::write(lib_path, new_content)?;
    Ok(())
}

fn create_basic_plugin_template(plugin_name: &str) -> String {
    let struct_name = to_pascal_case(plugin_name);
    
    format!(r#"use async_trait::async_trait;
use horizon_event_system::{{
    create_simple_plugin, current_timestamp, EventSystem, LogLevel,
    PluginError, ServerContext, SimplePlugin,
}};
use serde::{{Deserialize, Serialize}};
use std::sync::Arc;
use tracing::{{error, info, warn}};

/// {} Plugin
pub struct {}Plugin {{
    name: String,
}}

impl {}Plugin {{
    pub fn new() -> Self {{
        info!("🔧 {}Plugin: Creating new instance");
        Self {{
            name: "{}".to_string(),
        }}
    }}
}}

#[async_trait]
impl SimplePlugin for {}Plugin {{
    fn name(&self) -> &str {{
        &self.name
    }}

    fn version(&self) -> &str {{
        "1.0.0"
    }}

    async fn register_handlers(&mut self, _events: Arc<EventSystem>) -> Result<(), PluginError> {{
        info!("🔧 {}Plugin: Registering event handlers...");
        
        // TODO: Register your event handlers here
        // Example:
        // register_handlers!(events; core {{
        //     "your_event" => |event: serde_json::Value| {{
        //         info!("Received event: {{:?}}", event);
        //         Ok(())
        //     }}
        // }})?;
        
        info!("🔧 {}Plugin: ✅ All handlers registered successfully!");
        Ok(())
    }}

    async fn on_init(&mut self, context: Arc<dyn ServerContext>) -> Result<(), PluginError> {{
        context.log(
            LogLevel::Info,
            "🔧 {}Plugin: Starting up!",
        );

        // TODO: Add your initialization logic here
        
        info!("🔧 {}Plugin: ✅ Initialization complete!");
        Ok(())
    }}

    async fn on_shutdown(&mut self, context: Arc<dyn ServerContext>) -> Result<(), PluginError> {{
        context.log(
            LogLevel::Info,
            "🔧 {}Plugin: Shutting down!",
        );

        // TODO: Add your cleanup logic here

        info!("🔧 {}Plugin: ✅ Shutdown complete!");
        Ok(())
    }}
}}

// Create the plugin using the macro
create_simple_plugin!({}Plugin);
"#, 
        &struct_name,      // Comment
        &struct_name,      // struct name
        &struct_name,      // impl block
        &struct_name,      // log message
        plugin_name,       // name field
        &struct_name,      // trait impl
        &struct_name,      // register_handlers log
        &struct_name,      // register_handlers success log
        &struct_name,      // on_init log
        &struct_name,      // on_init success log
        &struct_name,      // on_shutdown log
        &struct_name,      // on_shutdown success log
        &struct_name       // macro call
    )
}

fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })
        .collect()
}

fn cleanup_plugin_directory(plugin_dir: &Path) -> Result<()> {
    // Remove .git directory
    let git_dir = plugin_dir.join(".git");
    if git_dir.exists() {
        fs::remove_dir_all(git_dir)?;
    }
    
    // Remove README if it exists (they can create their own)
    let readme_path = plugin_dir.join("README.md");
    if readme_path.exists() {
        fs::remove_file(readme_path)?;
    }
    
    Ok(())
}

async fn build_plugin(horizon_path: Option<PathBuf>, no_copy: bool, plugin: Option<String>) -> Result<()> {
    println!("🔨 Building Horizon plugin...");

    // Determine if we're in Horizon repo root or plugin crate dir
    let current_dir = std::env::current_dir()?;
    let cargo_toml = current_dir.join("Cargo.toml");
    let crates_dir = current_dir.join("crates");
    let in_plugin_dir = cargo_toml.exists();
    let in_horizon_root = crates_dir.exists();

    let (plugin_dir, package_name) = {
        // Use directory name for plugin detection, but use package name for DLL search
        let dir_name = current_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if dir_name == "plugin_system" {
            return Err(anyhow!("plugin_system is not a buildable plugin crate"));
        }
        if in_plugin_dir && dir_name.starts_with("plugin_") {
            println!("[DEBUG] Detected plugin crate by directory name: {}", dir_name);
            let cargo_toml_path = current_dir.join("Cargo.toml");
            let content = fs::read_to_string(&cargo_toml_path)?;
            let doc = content.parse::<Document>()?;
            let pkg_table = doc.get("package").and_then(|t| t.as_table());
            let pkg_name = pkg_table.and_then(|t| t.get("name")).and_then(|n| n.as_str());
            let pkg_name = match pkg_name {
                Some(name) => name.to_string(),
                None => {
                    return Err(anyhow!("Cargo.toml missing [package] name field ({}).", cargo_toml_path.display()));
                }
            };
            (current_dir.clone(), pkg_name)
        } else if in_horizon_root {
            let plugin_arg = plugin.ok_or_else(|| anyhow!("--plugin argument required when in Horizon repo root"))?;
            let mut crate_name = plugin_arg.clone();
            if !crate_name.starts_with("plugin_") {
                crate_name = format!("plugin_{}", crate_name);
            }
            if crate_name == "plugin_system" {
                return Err(anyhow!("plugin_system is not a buildable plugin crate"));
            }
            let plugin_path = crates_dir.join(&crate_name);
            if !plugin_path.exists() {
                return Err(anyhow!("Plugin crate '{}' not found in crates dir", crate_name));
            }
            println!("[DEBUG] Detected plugin crate by directory name: {}", crate_name);
            let cargo_toml_path = plugin_path.join("Cargo.toml");
            let content = fs::read_to_string(&cargo_toml_path)?;
            let doc = content.parse::<Document>()?;
            let pkg_table = doc.get("package").and_then(|t| t.as_table());
            let pkg_name = pkg_table.and_then(|t| t.get("name")).and_then(|n| n.as_str());
            let pkg_name = match pkg_name {
                Some(name) => name.to_string(),
                None => {
                    return Err(anyhow!("Cargo.toml missing [package] name field ({}).", cargo_toml_path.display()));
                }
            };
            (plugin_path, pkg_name)
        } else {
            return Err(anyhow!("Not in a plugin crate directory or Horizon repo root"));
        }
    };

    // Create progress bar
    let pb = ProgressBar::new(if no_copy { 2 } else { 3 });
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    // Step 1: Build the plugin
    pb.set_message("Building plugin (release mode)...");
    build_release_in_dir(&plugin_dir)?;
    pb.inc(1);

    // Step 2: Find the built library
    pb.set_message("Locating built library...");
    let lib_path = if in_horizon_root {
        // Built library is in workspace root target/release
        let workspace_target_dir = current_dir.join("target/release");
        find_built_library_in_workspace(&workspace_target_dir, &package_name)?
    } else {
        // Check for workspace root in parent directories
        let mut ancestor = plugin_dir.as_path();
        let mut workspace_root = None;
        while let Some(parent) = ancestor.parent() {
            let candidate = parent.join("Cargo.toml");
            if candidate.exists() {
                let content = fs::read_to_string(&candidate)?;
                if content.contains("[workspace]") {
                    workspace_root = Some(parent.to_path_buf());
                    break;
                }
            }
            ancestor = parent;
        }
        let target_dir = if let Some(root) = workspace_root {
            println!("[DEBUG] Found workspace root: {}", root.display());
            root.join("target/release")
        } else {
            plugin_dir.join("target/release")
        };
        find_built_library_in_workspace(&target_dir, &package_name)?
    };
    pb.inc(1);

    // Step 3: Copy to Horizon plugins directory (if not skipped)
    if !no_copy {
        pb.set_message("Copying to Horizon plugins directory...");
        let target_path = horizon_path.clone().unwrap_or_else(|| PathBuf::from("../Horizon"));
        copy_to_horizon_plugins(&lib_path, &target_path)?;
        pb.inc(1);
    }

    pb.finish_with_message("✅ Plugin built successfully!");

    println!();
    println!("{}", "🎉 Plugin built successfully!".green().bold());
    println!("📄 Library: {}", style(lib_path.display()).yellow());

    if !no_copy {
        let target_path = horizon_path.unwrap_or_else(|| PathBuf::from("../Horizon"));
        let plugins_dir = target_path.join("plugins");
        println!("📁 Copied to: {}", style(plugins_dir.display()).yellow());
    }

    println!();
    Ok(())
}

fn build_release_in_dir(dir: &Path) -> Result<()> {
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(dir)
        .output()
        .context("Failed to execute cargo build")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Cargo build failed:\n{}", error));
    }

    Ok(())
}

fn find_built_library_in_dir(plugin_dir: &Path, plugin_name: &str) -> Result<PathBuf> {
    let target_dir = plugin_dir.join("target/release");
    find_built_library_in_workspace(&target_dir, plugin_name)
}

fn find_built_library_in_workspace(target_dir: &Path, plugin_name: &str) -> Result<PathBuf> {
    if !target_dir.exists() {
        return Err(anyhow!("Release target directory not found for plugin {} ({}).", plugin_name, target_dir.display()));
    }
    // Look for library files with common extensions
    let extensions = if cfg!(target_os = "windows") {
        vec!["dll"]
    } else if cfg!(target_os = "macos") {
        vec!["dylib"]
    } else {
        vec!["so"]
    };
    for entry in WalkDir::new(&target_dir).max_depth(1) {
        let entry = entry?;
        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extensions.contains(&extension.to_string_lossy().as_ref()) {
                if let Some(file_name) = path.file_name() {
                    let name = file_name.to_string_lossy();
                    // Match plugin library name
                    if name.starts_with(plugin_name) || (name.starts_with("plugin_") && name.contains(&plugin_name)) {
                        return Ok(path.to_path_buf());
                    }
                }
            }
        }
    }
    Err(anyhow!("Could not find built plugin library in {} for plugin {}", target_dir.display(), plugin_name))
}

fn build_release() -> Result<()> {
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .output()
        .context("Failed to execute cargo build")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Cargo build failed:\n{}", error));
    }

    Ok(())
}

fn find_built_library() -> Result<PathBuf> {
    let target_dir = PathBuf::from("target/release");
    
    if !target_dir.exists() {
        return Err(anyhow!("Release target directory not found"));
    }

    // Look for library files with common extensions
    let extensions = if cfg!(target_os = "windows") {
        vec!["dll"]
    } else if cfg!(target_os = "macos") {
        vec!["dylib"]
    } else {
        vec!["so"]
    };

    for entry in WalkDir::new(&target_dir).max_depth(1) {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(extension) = path.extension() {
            if extensions.contains(&extension.to_string_lossy().as_ref()) {
                if let Some(file_name) = path.file_name() {
                    let name = file_name.to_string_lossy();
                    if name.starts_with("plugin_") || name.contains("plugin") {
                        return Ok(path.to_path_buf());
                    }
                }
            }
        }
    }

    Err(anyhow!("Could not find built plugin library in target/release"))
}

fn copy_to_horizon_plugins(lib_path: &Path, horizon_path: &Path) -> Result<()> {
    let plugins_dir = horizon_path.join("plugins");
    
    // Create plugins directory if it doesn't exist
    if !plugins_dir.exists() {
        fs::create_dir_all(&plugins_dir)
            .with_context(|| format!("Failed to create plugins directory: {}", plugins_dir.display()))?;
    }

    let file_name = lib_path.file_name()
        .ok_or_else(|| anyhow!("Invalid library file path"))?;
    
    let target_path = plugins_dir.join(file_name);
    
    fs::copy(lib_path, &target_path)
        .with_context(|| format!("Failed to copy plugin to {}", target_path.display()))?;

    Ok(())
}