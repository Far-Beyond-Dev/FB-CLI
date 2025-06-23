use clap::Subcommand;
use anyhow::{anyhow, Context, Result};
use colored::*;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use git2::Repository;

const GITHUB_ORG: &str = "Far-Beyond-Dev";
const GITHUB_API_BASE: &str = "https://api.github.com";

#[derive(Subcommand)]
pub enum RepoCommand {
    /// List all repositories in the Far-Beyond-Dev organization
    List {
        /// Show only public repositories
        #[arg(long)]
        public_only: bool,
    },
    /// Clone a repository from Far-Beyond-Dev
    Clone {
        /// Repository name
        repo: String,
        /// Target directory (defaults to repo name)
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Use SSH instead of HTTPS
        #[arg(long)]
        ssh: bool,
    },
    /// Update all Far-Beyond repositories in current directory
    Update {
        /// Perform a dry run (show what would be updated)
        #[arg(long)]
        dry_run: bool,
    },
    /// Check status of all Far-Beyond repositories
    Status,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubRepo {
    name: String,
    full_name: String,
    description: Option<String>,
    html_url: String,
    clone_url: String,
    ssh_url: String,
    private: bool,
    default_branch: String,
    updated_at: String,
}

pub async fn handle_command(cmd: RepoCommand) -> Result<()> {
    match cmd {
        RepoCommand::List { public_only } => list_repositories(public_only).await,
        RepoCommand::Clone { repo, path, ssh } => clone_repository(&repo, path, ssh).await,
        RepoCommand::Update { dry_run } => update_repositories(dry_run).await,
        RepoCommand::Status => check_repository_status().await,
    }
}

async fn list_repositories(public_only: bool) -> Result<()> {
    println!("📋 Fetching repositories from {}...", style(GITHUB_ORG).cyan().bold());

    let client = reqwest::Client::new();
    let url = format!("{}/orgs/{}/repos?per_page=100&type=all", GITHUB_API_BASE, GITHUB_ORG);
    
    let response = client
        .get(&url)
        .header("User-Agent", "fbcli")
        .send()
        .await
        .context("Failed to fetch repositories from GitHub")?;

    if !response.status().is_success() {
        return Err(anyhow!("GitHub API request failed: {}", response.status()));
    }

    let repos: Vec<GitHubRepo> = response
        .json()
        .await
        .context("Failed to parse GitHub API response")?;

    let filtered_repos: Vec<&GitHubRepo> = repos
        .iter()
        .filter(|repo| !public_only || !repo.private)
        .collect();

    println!();
    println!("{} Found {} repositories:", "📦".bold(), filtered_repos.len());
    println!();

    for repo in filtered_repos {
        let visibility = if repo.private { "🔒 Private" } else { "🌐 Public" };
        let description = repo.description.as_deref().unwrap_or("No description");
        
        println!("{} {}", "▶".bright_blue(), style(&repo.name).cyan().bold());
        println!("  {} {}", visibility, style(description).dim());
        println!("  🔗 {}", style(&repo.html_url).blue().underlined());
        println!();
    }

    Ok(())
}

async fn clone_repository(repo_name: &str, target_path: Option<PathBuf>, use_ssh: bool) -> Result<()> {
    let target_dir = target_path.unwrap_or_else(|| PathBuf::from(repo_name));
    
    println!("📥 Cloning repository: {}", style(repo_name).cyan().bold());
    println!("📂 Target directory: {}", style(target_dir.display()).yellow());

    if target_dir.exists() {
        return Err(anyhow!("Directory '{}' already exists", target_dir.display()));
    }

    let repo_url = if use_ssh {
        format!("git@github.com:{}/{}.git", GITHUB_ORG, repo_name)
    } else {
        format!("https://github.com/{}/{}.git", GITHUB_ORG, repo_name)
    };

    println!("🔗 Repository URL: {}", style(&repo_url).blue());

    // Create progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    pb.set_message("Cloning repository...");

    // Clone the repository
    let result = Repository::clone(&repo_url, &target_dir);
    pb.finish_and_clear();

    match result {
        Ok(_) => {
            println!("{}", "✅ Repository cloned successfully!".green().bold());
            println!("📁 Location: {}", style(target_dir.display()).yellow());
            
            // Show next steps
            println!();
            println!("{}", "Next steps:".bold());
            println!("  cd {}", repo_name);
            
            // Check if it's a Rust project
            if target_dir.join("Cargo.toml").exists() {
                println!("  cargo build");
            }
        },
        Err(e) => {
            return Err(anyhow!("Failed to clone repository: {}", e));
        }
    }

    Ok(())
}

async fn update_repositories(dry_run: bool) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    
    println!("🔄 Scanning for Far-Beyond repositories in: {}", style(current_dir.display()).yellow());
    
    let mut repos_found = Vec::new();
    
    // Scan for git repositories
    for entry in fs::read_dir(&current_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let git_dir = path.join(".git");
            if git_dir.exists() {
                // Check if it's a Far-Beyond repository
                if let Ok(repo) = Repository::open(&path) {
                    if let Ok(remote) = repo.find_remote("origin") {
                        if let Some(url) = remote.url() {
                            if url.contains("Far-Beyond-Dev") || url.contains("far-beyond-dev") {
                                repos_found.push(path);
                            }
                        }
                    }
                }
            }
        }
    }

    if repos_found.is_empty() {
        println!("❌ No Far-Beyond repositories found in current directory");
        return Ok(());
    }

    println!("📦 Found {} Far-Beyond repositories:", repos_found.len());
    
    for repo_path in &repos_found {
        let repo_name = repo_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        println!("  • {}", style(repo_name).cyan());
    }
    
    if dry_run {
        println!();
        println!("{}", "🔍 Dry run mode - no changes will be made".yellow().bold());
        return Ok(());
    }

    println!();
    println!("🔄 Updating repositories...");

    for repo_path in repos_found {
        let repo_name = repo_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        print!("  Updating {}... ", style(repo_name).cyan());
        
        match update_single_repository(&repo_path) {
            Ok(updated) => {
                if updated {
                    println!("{}", "✅ Updated".green());
                } else {
                    println!("{}", "📋 Already up to date".blue());
                }
            },
            Err(e) => {
                println!("{} {}", "❌ Failed:".red(), e);
            }
        }
    }

    println!();
    println!("{}", "✅ Repository update complete!".green().bold());
    Ok(())
}

fn update_single_repository(repo_path: &Path) -> Result<bool> {
    let repo = Repository::open(repo_path)?;    // Fetch from origin
    let mut remote = repo.find_remote("origin")?;
    let refspecs: &[&str] = &[];
    remote.fetch(refspecs, None, None)?;
    
    // Get current branch
    let head = repo.head()?;
    let local_oid = head.target().unwrap();
    
    // Get remote branch
    let branch_name = head.shorthand().unwrap_or("main");
    let remote_branch_name = format!("origin/{}", branch_name);
    let remote_ref = repo.find_reference(&format!("refs/remotes/{}", remote_branch_name))?;
    let remote_oid = remote_ref.target().unwrap();
    
    // Check if update is needed
    if local_oid == remote_oid {
        return Ok(false); // Already up to date
    }
    
    // Perform fast-forward merge
    let (analysis, _) = repo.merge_analysis(&[&repo.find_annotated_commit(remote_oid)?])?;
    
    if analysis.is_fast_forward() {
        // Update the reference
        let mut reference = repo.find_reference(&format!("refs/heads/{}", branch_name))?;
        reference.set_target(remote_oid, "Fast-forward")?;
        
        // Update working directory
        repo.set_head(&format!("refs/heads/{}", branch_name))?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        
        Ok(true)
    } else {
        Err(anyhow!("Cannot fast-forward, manual merge required"))
    }
}

async fn check_repository_status() -> Result<()> {
    let current_dir = std::env::current_dir()?;
    
    println!("📊 Checking status of Far-Beyond repositories...");
    println!("📂 Scanning directory: {}", style(current_dir.display()).yellow());
    println!();

    let mut repos_found = 0;
    
    for entry in fs::read_dir(&current_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let git_dir = path.join(".git");
            if git_dir.exists() {
                if let Ok(repo) = Repository::open(&path) {
                    if let Ok(remote) = repo.find_remote("origin") {
                        if let Some(url) = remote.url() {
                            if url.contains("Far-Beyond-Dev") || url.contains("far-beyond-dev") {
                                repos_found += 1;
                                show_repository_status(&path, &repo)?;
                                println!();
                            }
                        }
                    }
                }
            }
        }
    }

    if repos_found == 0 {
        println!("❌ No Far-Beyond repositories found in current directory");
    } else {
        println!("📈 Status check complete for {} repositories", repos_found);
    }

    Ok(())
}

fn show_repository_status(repo_path: &Path, repo: &Repository) -> Result<()> {
    let repo_name = repo_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    
    println!("{} {}", "📦".bold(), style(repo_name).cyan().bold());
    
    // Current branch
    if let Ok(head) = repo.head() {
        if let Some(branch_name) = head.shorthand() {
            println!("  🌿 Branch: {}", style(branch_name).green());
        }
    }
    
    // Check for uncommitted changes
    let statuses = repo.statuses(None)?;
    let mut changes = Vec::new();
    
    for status in statuses.iter() {
        let flags = status.status();
        if flags.contains(git2::Status::WT_MODIFIED) {
            changes.push("modified");
        }
        if flags.contains(git2::Status::WT_NEW) {
            changes.push("untracked");
        }
        if flags.contains(git2::Status::INDEX_NEW) {
            changes.push("staged");
        }
    }
    
    if changes.is_empty() {
        println!("  ✅ Working directory clean");
    } else {
        println!("  ⚠️  Uncommitted changes: {}", changes.join(", "));
    }
    
    // Check if behind/ahead of remote
    if let Ok(head) = repo.head() {
        if let Some(branch_name) = head.shorthand() {
            let remote_branch_name = format!("origin/{}", branch_name);
            if let Ok(remote_ref) = repo.find_reference(&format!("refs/remotes/{}", remote_branch_name)) {
                let local_oid = head.target().unwrap();
                let remote_oid = remote_ref.target().unwrap();
                
                if local_oid != remote_oid {
                    let (ahead, behind) = repo.graph_ahead_behind(local_oid, remote_oid)?;
                    if ahead > 0 {
                        println!("  ⬆️  {} commits ahead", ahead);
                    }
                    if behind > 0 {
                        println!("  ⬇️  {} commits behind", behind);
                    }
                } else {
                    println!("  🔄 Up to date with remote");
                }
            }
        }
    }
    
    Ok(())
}