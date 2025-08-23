use clap::{ Parser, Subcommand };
use std::process::Command;
use std::env;
use std::path::{Path, PathBuf};

// For symbolic links on Linux
#[cfg(target_os = "linux")]
use std::os::unix::fs::symlink;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "A powerful PHP version manager for Linux.",
    long_about = "Easily switch between PHP versions on your Linux system with this awesome tool! 🚀"
)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 📦 Installs a specific version of PHP
    Install {
        /// The PHP version to install (e.g., 8.1.10)
        version: String,
    },
    /// ✨ Sets the global PHP version to use
    Use {
        /// The PHP version to use
        version: String,
    },
    /// 📜 Lists all installed PHP versions
    List,
    /// 🔍 Displays the path to the currently active PHP binary
    Which,
    /// 🌐 Shows available PHP versions from official website
    Available {
        /// Required: filter by major version (e.g., 7, 8, 8.1)
        version: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Install { version } => {
            if let Err(e) = install_php(version) {
                eprintln!("🔴 Error installing PHP: {}", e);
            }
        }
        Commands::Use { version } => {
            if let Err(e) = use_php(version) {
                eprintln!("🔴 Error switching PHP version: {}", e);
            }
        }
        Commands::List => {
            println!("✨ List of installed PHP versions: ✨");
            if let Err(e) = list_installed_versions() {
                eprintln!("🔴 Error listing installed versions: {}", e);
            }
        }
        Commands::Which => {
            let output = Command::new("which").arg("php").output().expect("Failed to execute command");
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        Commands::Available { version } => {
            if version.is_none() {
                eprintln!("❌ Sorry, you must provide a version prefix to search for.");
                return;
            }
            println!("🌐 Fetching available PHP versions from official website...");
            if let Err(e) = show_available_versions(version) {
                eprintln!("🔴 Error fetching available versions: {}", e);
            }
        }
    }
}

// Function to show available PHP versions
fn show_available_versions(filter: &Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("curl")
        .arg("-s")
        .arg("https://www.php.net/releases/")
        .output()?;

    if !output.status.success() {
        return Err("Failed to fetch PHP releases page".into());
    }

    let html = String::from_utf8_lossy(&output.stdout);
    let mut versions = Vec::new();

    for line in html.lines() {
        if line.contains("php-") && line.contains(".tar.gz") {
            if let Some(start) = line.find("php-") {
                let start_idx = start + 4;
                if let Some(end) = line[start_idx..].find(".tar.gz") {
                    let version = &line[start_idx..start_idx + end];
                    if version.contains('.') && version.chars().any(|c| c.is_numeric()) {
                        versions.push(version.to_string());
                    }
                }
            }
        }
    }

    versions.sort_by(|a, b| {
        let a_parts: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
        let b_parts: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();
        b_parts.cmp(&a_parts)
    });
    versions.dedup();

    let active_versions = vec!["8.3", "8.2"];
    let lts_versions = vec!["8.1"];

    if versions.is_empty() {
        println!("⚠️  Could not fetch available versions.");
        return Ok(());
    }

    if let Some(filter_str) = filter {
        println!("📋 Available PHP versions for {}:", filter_str);
        let prefix = if filter_str.contains('.') {
            format!("{}.", filter_str)
        } else {
            format!("{}.", filter_str)
        };

        let filtered: Vec<_> = versions.iter()
            .filter(|v| v.starts_with(&prefix))
            .collect();

        if filtered.is_empty() {
            println!("⚠️  No versions found for {}", filter_str);
        } else {
            for version in filtered {
                let short = version.split('.').take(2).collect::<Vec<_>>().join(".");
                if active_versions.contains(&short.as_str()) {
                    println!("   • {} ⚡ (Active)", version);
                } else if lts_versions.contains(&short.as_str()) {
                    println!("   • {} 🔒 (LTS)", version);
                } else {
                    println!("   • {} ☠️ (EOL)", version);
                }
            }
        }
    }

    println!("\nℹ️  Legend: ⚡ Active | 🔒 LTS | ☠️ EOL");
    println!("💡 Use 'palawija install <version>' to install a specific version");
    Ok(())
}

// Function to list installed PHP versions
fn list_installed_versions() -> Result<(), Box<dyn std::error::Error>> {
    let home = env::var("HOME")?;
    let install_dir = PathBuf::from(format!("{}/.palawija", home));

    if !install_dir.exists() {
        println!("📭 No PHP versions installed yet.");
        println!("💡 Use 'palawija available <version>' to see available versions");
        println!("💡 Use 'palawija install <version>' to install a version");
        return Ok(());
    }

    let mut installed_versions = Vec::new();

    for entry in std::fs::read_dir(&install_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    if name_str.starts_with("php-") {
                        let version = &name_str[4..];
                        installed_versions.push(version.to_string());
                    }
                }
            }
        }
    }

    if installed_versions.is_empty() {
        println!("📭 No PHP versions installed yet.");
    } else {
        installed_versions.sort();
        for version in installed_versions {
            let php_bin_path = install_dir.join(format!("php-{}", version)).join("bin").join("php");
            let is_active = if php_bin_path.exists() {
                if let Ok(output) = Command::new("readlink").arg("/usr/local/bin/php").output() {
                    let current_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    current_path == php_bin_path.to_string_lossy()
                } else {
                    false
                }
            } else {
                false
            };

            if is_active {
                println!("   • {} ⭐ (active)", version);
            } else {
                println!("   • {}", version);
            }
        }
    }

    Ok(())
}

fn install_php(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("⚙️  Installing PHP version: {}", version);

    let install_dir = format!("{}/.palawija", env::var("HOME")?);
    std::fs::create_dir_all(&install_dir)?;

    let version_dir = format!("{}/php-{}", install_dir, version);
    if Path::new(&version_dir).exists() {
        println!("⚠️  PHP version {} is already installed!", version);
        println!("💡 Use 'palawija use {}' to switch to this version", version);
        return Ok(());
    }

    let php_url = format!("https://www.php.net/distributions/php-{}.tar.gz", version);
    println!("⬇️ Downloading PHP from {}...", php_url);
    let tar_gz_path = format!("{}/php-{}.tar.gz", install_dir, version);

    let download_result = Command::new("curl")
        .arg("-L")
        .arg("-f")
        .arg(&php_url)
        .arg("-o")
        .arg(&tar_gz_path)
        .status()?;

    if !download_result.success() {
        return Err(format!("Failed to download PHP version {}. Please check if the version exists.", version).into());
    }
    
    println!("📂 Extracting files to {}...", install_dir);
    let extracted_dir = format!("{}/php-{}", install_dir, version);
    std::fs::create_dir_all(&extracted_dir)?;

    Command::new("tar")
        .arg("-xzf")
        .arg(&tar_gz_path)
        .arg("-C")
        .arg(&extracted_dir)
        .arg("--strip-components=1")
        .status()?;
    
    std::fs::remove_file(&tar_gz_path)?;

    println!("✅ PHP extracted to {}", extracted_dir);
    println!("⚠️  Note: This extracts the source code. You may need to compile it:");
    println!("   cd {}", extracted_dir);
    println!("   ./configure --prefix={}/bin", extracted_dir);
    println!("   make && make install");

    println!("\n🎉 PHP version {} downloaded successfully! Time to compile and code! 💻🎉", version);
    Ok(())
}

fn use_php(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Setting default PHP version to: {}", version);

    let home = env::var("HOME")?;
    let install_dir = PathBuf::from(format!("{}/.palawija", home));

    let php_bin_path = install_dir.join(format!("php-{}", version)).join("bin").join("php");

    if !php_bin_path.exists() {
        return Err(format!("PHP version {} is not installed at {:?}. Use 'palawija install {}' first.", version, php_bin_path, version).into());
    }

    let link_path = Path::new("/usr/local/bin/php");
    if link_path.exists() {
        std::fs::remove_file(link_path)?;
    }
    
    symlink(&php_bin_path, &link_path)?;
    
    println!("\n✅ PHP version {} is now your default. Happy coding! 🤩✅", version);
    Ok(())
}
