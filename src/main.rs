/*!
 * Palawija - PHP Version Manager for Linux
 * 
 * A powerful and user-friendly PHP version manager that allows you to easily
 * install, switch between, and manage multiple PHP versions on your Linux system.
 * 
 * Features:
 * - 📦 Install PHP versions from official sources
 * - ✨ Switch between installed PHP versions seamlessly  
 * - 📜 List all installed PHP versions with status indicators
 * - 🔍 Show currently active PHP binary path
 * - 🌐 Browse available PHP versions from php.net
 * 
 * Author: Your Name
 * Version: 1.0.0
 * License: MIT
 */

use clap::{ Parser, Subcommand };
use std::process::Command;
use std::env;
use std::path::{Path, PathBuf};

// For symbolic links on Linux - required for the 'use' command
#[cfg(target_os = "linux")]
use std::os::unix::fs::symlink;

/// Main CLI structure using clap derive macros
#[derive(Parser)]
#[command(
    author = "Palawija Team",
    version = "1.0.0",
    about = "🚀 A powerful PHP version manager for Linux systems",
    long_about = "Palawija makes it incredibly easy to install, manage, and switch between \
                 multiple PHP versions on your Linux system. Perfect for developers who \
                 work with different PHP projects requiring different versions! 🎯✨"
)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands for the PHP version manager
#[derive(Subcommand)]
enum Commands {
    /// 📦 Install a specific PHP version from official sources
    #[command(about = "Downloads and extracts PHP source code for compilation")]
    Install {
        /// The PHP version to install (e.g., 8.3.0, 8.2.15, 7.4.33)
        #[arg(help = "PHP version in format: major.minor.patch (e.g., 8.3.0)")]
        version: String,
    },
    
    /// ✨ Switch to a different installed PHP version as the system default
    #[command(about = "Sets the global PHP version by creating symbolic links")]
    Use {
        /// The PHP version to use (must be already installed)
        #[arg(help = "Previously installed PHP version to switch to")]
        version: String,
    },
    
    /// 📜 Display all installed PHP versions with their status
    #[command(about = "Shows installed versions and highlights the currently active one")]
    List,
    
    /// 🔍 Show the path to the currently active PHP binary
    #[command(about = "Displays the full path to the current PHP executable")]
    Which,
    
    /// 🌐 Browse available PHP versions from the official website
    #[command(about = "Fetches and displays available PHP versions with their status")]
    Available {
        /// Filter by major version (e.g., 7, 8, 8.1, 8.2)
        #[arg(help = "Version prefix to filter results (e.g., '8' for PHP 8.x, '8.2' for 8.2.x)")]
        version: Option<String>,
    },
}

/// Application entry point - parses CLI arguments and dispatches to appropriate handlers
fn main() {
    println!("🎯 Palawija PHP Version Manager v1.0.0");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let cli = Cli::parse();

    // Match and execute the appropriate command
    match &cli.command {
        Commands::Install { version } => {
            println!("🚀 Starting PHP installation process...\n");
            if let Err(e) = install_php(version) {
                eprintln!("❌ Installation failed: {}", e);
                eprintln!("💡 Tip: Ensure you have internet connection and sufficient disk space");
                std::process::exit(1);
            }
        }
        
        Commands::Use { version } => {
            println!("🔄 Switching PHP version...\n");
            if let Err(e) = use_php(version) {
                eprintln!("❌ Failed to switch PHP version: {}", e);
                eprintln!("💡 Tip: Make sure the version is installed first using 'palawija install {}'", version);
                std::process::exit(1);
            }
        }
        
        Commands::List => {
            println!("📋 Scanning for installed PHP versions...\n");
            if let Err(e) = list_installed_versions() {
                eprintln!("❌ Error while listing versions: {}", e);
                std::process::exit(1);
            }
        }
        
        Commands::Which => {
            println!("🔍 Locating current PHP binary...\n");
            match Command::new("which").arg("php").output() {
                Ok(output) => {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if path.is_empty() {
                        println!("⚠️  No PHP binary found in system PATH");
                        println!("💡 Install a PHP version with: palawija install <version>");
                        println!("💡 Then set it as default with: palawija use <version>");
                    } else {
                        println!("📍 Current PHP binary location:");
                        println!("   {}", path);
                        
                        // Try to get PHP version info
                        if let Ok(version_output) = Command::new("php").arg("--version").output() {
                            let version_info = String::from_utf8_lossy(&version_output.stdout);
                            if let Some(first_line) = version_info.lines().next() {
                                println!("ℹ️  Version info: {}", first_line);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to locate PHP binary: {}", e);
                }
            }
        }
        
        Commands::Available { version } => {
            if version.is_none() {
                eprintln!("❌ Missing required parameter!");
                eprintln!("📝 Usage: palawija available <version-prefix>");
                eprintln!("📝 Examples:");
                eprintln!("   palawija available 8     # Show all PHP 8.x versions");
                eprintln!("   palawija available 8.2   # Show all PHP 8.2.x versions");
                eprintln!("   palawija available 7.4   # Show all PHP 7.4.x versions");
                std::process::exit(1);
            }
            println!("🌐 Fetching available PHP versions from official website...\n");
            if let Err(e) = show_available_versions(version) {
                eprintln!("❌ Failed to fetch available versions: {}", e);
                eprintln!("💡 Check your internet connection and try again");
                std::process::exit(1);
            }
        }
    }
}

/**
 * Fetches and displays available PHP versions from php.net
 * 
 * This function scrapes the PHP releases page to get available versions,
 * sorts them by version number, and displays them with status indicators.
 * 
 * # Arguments
 * * `filter` - Optional version prefix to filter results (e.g., "8", "8.2")
 * 
 * # Returns
 * * `Result<(), Box<dyn std::error::Error>>` - Success or error details
 * 
 * # Status Indicators
 * * ⚡ Active - Currently supported and actively developed
 * * 🔒 LTS - Long Term Support, recommended for production
 * * ☠️ EOL - End of Life, no longer supported
 */
fn show_available_versions(filter: &Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("📡 Connecting to https://www.php.net/releases/...");
    
    let output = Command::new("curl")
        .arg("-s")              // Silent mode
        .arg("-L")              // Follow redirects
        .arg("--max-time")      // Set timeout
        .arg("30")
        .arg("https://www.php.net/releases/")
        .output()?;

    if !output.status.success() {
        return Err("🌐 Failed to fetch PHP releases page. Check your internet connection.".into());
    }

    println!("✅ Successfully retrieved releases page");
    println!("🔍 Parsing available versions...\n");

    let html = String::from_utf8_lossy(&output.stdout);
    let mut versions = Vec::new();

    // Parse HTML to extract PHP version numbers
    for line in html.lines() {
        if line.contains("php-") && line.contains(".tar.gz") {
            if let Some(start) = line.find("php-") {
                let start_idx = start + 4;
                if let Some(end) = line[start_idx..].find(".tar.gz") {
                    let version = &line[start_idx..start_idx + end];
                    // Validate version format (should contain dots and numbers)
                    if version.contains('.') && version.chars().any(|c| c.is_numeric()) {
                        versions.push(version.to_string());
                    }
                }
            }
        }
    }

    // Sort versions in descending order (newest first)
    versions.sort_by(|a, b| {
        let a_parts: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
        let b_parts: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();
        b_parts.cmp(&a_parts)
    });
    versions.dedup(); // Remove duplicates

    // PHP version status definitions (as of 2024)
    let active_versions = vec!["8.3", "8.2"];     // Currently active branches
    let lts_versions = vec!["8.1"];               // Long Term Support
    // Everything else is considered EOL (End of Life)

    if versions.is_empty() {
        println!("⚠️  Could not parse any versions from the releases page.");
        println!("🔄 The website format might have changed. Please try again later.");
        return Ok(());
    }

    // Display filtered results
    if let Some(filter_str) = filter {
        println!("🎯 Available PHP versions matching '{}':", filter_str);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        // Create filter prefix (ensure it ends with dot for proper matching)
        let prefix = if filter_str.contains('.') {
            format!("{}.", filter_str)
        } else {
            format!("{}.", filter_str)
        };

        let filtered: Vec<_> = versions.iter()
            .filter(|v| v.starts_with(&prefix))
            .collect();

        if filtered.is_empty() {
            println!("😔 No versions found matching '{}'", filter_str);
            println!("💡 Try a broader search like 'palawija available 8' or 'palawija available 7'");
        } else {
            println!("📊 Found {} matching versions:\n", filtered.len());
            
            for version in filtered {
                // Extract major.minor for status checking
                let short = version.split('.').take(2).collect::<Vec<_>>().join(".");
                
                // Display version with appropriate status indicator
                if active_versions.contains(&short.as_str()) {
                    println!("   📦 {} ⚡ (Active - Recommended)", version);
                } else if lts_versions.contains(&short.as_str()) {
                    println!("   📦 {} 🔒 (LTS - Stable)", version);
                } else {
                    println!("   📦 {} ☠️  (EOL - Not Recommended)", version);
                }
            }
        }
    }

    // Display legend and usage instructions
    println!("\n📚 Status Legend:");
    println!("   ⚡ Active    - Latest stable versions with active development");
    println!("   🔒 LTS       - Long Term Support, perfect for production");
    println!("   ☠️  EOL       - End of Life, security updates discontinued");
    
    println!("\n💡 Usage Examples:");
    println!("   palawija install 8.3.0    # Install latest PHP 8.3");
    println!("   palawija install 8.2.15   # Install specific PHP 8.2 version");
    println!("   palawija use 8.3.0        # Switch to PHP 8.3.0");
    
    Ok(())
}

/**
 * Lists all installed PHP versions in the user's home directory
 * 
 * Scans ~/.palawija directory for installed PHP versions and displays them
 * with indicators showing which version is currently active.
 * 
 * # Returns
 * * `Result<(), Box<dyn std::error::Error>>` - Success or error details
 */
fn list_installed_versions() -> Result<(), Box<dyn std::error::Error>> {
    let home = env::var("HOME")?;
    let install_dir = PathBuf::from(format!("{}/.palawija", home));

    println!("📂 Scanning installation directory: ~/.palawija");

    if !install_dir.exists() {
        println!("📭 No PHP versions installed yet.\n");
        println!("🚀 Getting Started:");
        println!("   1. Check available versions: palawija available 8");
        println!("   2. Install a version:        palawija install 8.3.0");  
        println!("   3. Set as default:           palawija use 8.3.0");
        return Ok(());
    }

    let mut installed_versions = Vec::new();

    // Scan for installed PHP directories
    for entry in std::fs::read_dir(&install_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    if name_str.starts_with("php-") {
                        let version = &name_str[4..];  // Remove "php-" prefix
                        installed_versions.push(version.to_string());
                    }
                }
            }
        }
    }

    if installed_versions.is_empty() {
        println!("📭 Installation directory exists but no PHP versions found.\n");
        println!("💡 Try installing a PHP version:");
        println!("   palawija available 8    # Browse available versions");
        println!("   palawija install 8.3.0  # Install PHP 8.3.0");
    } else {
        // Sort versions for better display
        installed_versions.sort();
        
        println!("✅ Found {} installed PHP version(s):", installed_versions.len());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        for version in installed_versions {
            let php_bin_path = install_dir.join(format!("php-{}", version)).join("bin").join("php");
            
            // Check if this version is currently active by examining the symlink
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

            // Display version with status indicator
            if is_active {
                println!("   📦 {} ⭐ (Currently Active)", version);
            } else {
                // Check if the binary actually exists (compiled)
                if php_bin_path.exists() {
                    println!("   📦 {} ✅ (Ready to use)", version);
                } else {
                    println!("   📦 {} ⚠️  (Source only - needs compilation)", version);
                }
            }
        }
        
        println!("\n💡 Management Commands:");
        println!("   palawija use <version>     # Switch to a different version");
        println!("   palawija which             # Show current PHP binary path");
        println!("   php --version              # Check active PHP version");
    }

    Ok(())
}

/**
 * Downloads and extracts PHP source code for a specific version
 * 
 * This function downloads the official PHP source tarball from php.net,
 * extracts it to ~/.palawija/php-<version>/, and provides compilation instructions.
 * 
 * Note: This only downloads and extracts source code. The user needs to compile
 * it manually using the standard ./configure && make && make install process.
 * 
 * # Arguments
 * * `version` - PHP version string (e.g., "8.3.0", "8.2.15")
 * 
 * # Returns
 * * `Result<(), Box<dyn std::error::Error>>` - Success or error details
 */
fn install_php(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Target PHP version: {}", version);
    
    // Validate version format (basic check)
    if !version.contains('.') || !version.chars().any(|c| c.is_numeric()) {
        return Err("❌ Invalid version format. Use format like '8.3.0' or '8.2.15'".into());
    }

    let install_dir = format!("{}/.palawija", env::var("HOME")?);
    println!("📁 Installation directory: {}", install_dir);
    
    // Create installation directory if it doesn't exist
    std::fs::create_dir_all(&install_dir)?;
    println!("✅ Installation directory ready");

    let version_dir = format!("{}/php-{}", install_dir, version);
    
    // Check if version already exists
    if Path::new(&version_dir).exists() {
        println!("⚠️  PHP version {} is already downloaded!", version);
        println!("📂 Location: {}", version_dir);
        println!("💡 To use this version: palawija use {}", version);
        
        // Check if it's compiled
        let binary_path = Path::new(&version_dir).join("bin").join("php");
        if binary_path.exists() {
            println!("✅ Binary found - ready to use!");
        } else {
            println!("⚙️  Source code only - compilation required");
            print_compilation_instructions(&version_dir);
        }
        return Ok(());
    }

    // Download PHP source code
    let php_url = format!("https://www.php.net/distributions/php-{}.tar.gz", version);
    println!("🌐 Download URL: {}", php_url);
    println!("⬇️  Starting download...");
    
    let tar_gz_path = format!("{}/php-{}.tar.gz", install_dir, version);

    let download_result = Command::new("curl")
        .arg("-L")              // Follow redirects
        .arg("-f")              // Fail on HTTP errors
        .arg("--progress-bar")  // Show progress bar
        .arg("--max-time")      // Set timeout
        .arg("300")             // 5 minutes timeout
        .arg(&php_url)
        .arg("-o")
        .arg(&tar_gz_path)
        .status()?;

    if !download_result.success() {
        // Clean up partial download
        let _ = std::fs::remove_file(&tar_gz_path);
        return Err(format!(
            "❌ Download failed for PHP version {}.\n💡 Possible reasons:\n   • Version doesn't exist\n   • Network connection issues\n   • Server temporarily unavailable", 
            version
        ).into());
    }
    
    println!("✅ Download completed successfully");
    
    // Extract the tarball
    println!("📦 Extracting source code...");
    let extracted_dir = format!("{}/php-{}", install_dir, version);
    std::fs::create_dir_all(&extracted_dir)?;

    let extract_result = Command::new("tar")
        .arg("-xzf")
        .arg(&tar_gz_path)
        .arg("-C")
        .arg(&extracted_dir)
        .arg("--strip-components=1")  // Remove top-level directory
        .status()?;
    
    if !extract_result.success() {
        return Err("❌ Failed to extract PHP source code".into());
    }
    
    // Clean up downloaded tarball
    std::fs::remove_file(&tar_gz_path)?;
    println!("✅ Source code extracted to: {}", extracted_dir);
    println!("🗑️  Cleaned up download archive");

    // Provide compilation instructions
    print_compilation_instructions(&extracted_dir);

    println!("\n🎉 PHP {} source code ready for compilation!", version);
    println!("📝 After successful compilation, use: palawija use {}", version);
    
    Ok(())
}

/**
 * Prints detailed compilation instructions for PHP source code
 * 
 * # Arguments  
 * * `source_dir` - Path to the extracted PHP source directory
 */
fn print_compilation_instructions(source_dir: &str) {
    println!("\n⚙️  Compilation Instructions:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 Step-by-step compilation process:");
    println!("");
    println!("1️⃣  Navigate to source directory:");
    println!("   cd {}", source_dir);
    println!("");
    println!("2️⃣  Configure build (basic configuration):");
    println!("   ./configure \\");
    println!("     --prefix={}/bin \\", source_dir);
    println!("     --with-config-file-path={}/etc \\", source_dir);
    println!("     --enable-mbstring \\");
    println!("     --enable-zip \\");
    println!("     --with-curl \\");
    println!("     --with-openssl \\");
    println!("     --with-zlib \\");
    println!("     --enable-soap");
    println!("");
    println!("3️⃣  Compile (this may take 10-30 minutes):");
    println!("   make -j$(nproc)");
    println!("");
    println!("4️⃣  Install:");
    println!("   make install");
    println!("");
    println!("📝 Note: You may need to install development packages:");
    println!("   # Ubuntu/Debian:");
    println!("   sudo apt-get install build-essential libxml2-dev libssl-dev libcurl4-openssl-dev");
    println!("   # CentOS/RHEL/Fedora:");
    println!("   sudo yum install gcc libxml2-devel openssl-devel curl-devel");
}

/**
 * Switches the system default PHP version by creating symbolic links
 * 
 * This function creates a symbolic link from /usr/local/bin/php to the
 * specified PHP version's binary, making it the system default.
 * 
 * # Arguments
 * * `version` - The PHP version to switch to (must be compiled and installed)
 * 
 * # Returns
 * * `Result<(), Box<dyn std::error::Error>>` - Success or error details
 * 
 * # Security Note
 * This function requires write permissions to /usr/local/bin/ which typically
 * requires sudo privileges or proper user permissions.
 */
fn use_php(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Target version: {}", version);

    let home = env::var("HOME")?;
    let install_dir = PathBuf::from(format!("{}/.palawija", home));

    // Construct path to the PHP binary
    let php_bin_path = install_dir
        .join(format!("php-{}", version))
        .join("bin")
        .join("php");

    println!("🔍 Looking for PHP binary at: {}", php_bin_path.display());

    // Verify the PHP binary exists and is executable
    if !php_bin_path.exists() {
        println!("❌ PHP version {} not found!", version);
        println!("📂 Expected location: {}", php_bin_path.display());
        println!("");
        println!("🔧 Possible solutions:");
        println!("   1. Install the version: palawija install {}", version);
        println!("   2. Check installed versions: palawija list");
        println!("   3. Verify compilation completed successfully");
        
        return Err(format!("PHP binary not found for version {}", version).into());
    }

    // Test if the binary is actually executable
    match Command::new(&php_bin_path).arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                let version_info = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = version_info.lines().next() {
                    println!("✅ Found working PHP binary: {}", first_line.trim());
                }
            } else {
                println!("⚠️  PHP binary exists but may not be working properly");
            }
        }
        Err(_) => {
            println!("⚠️  Could not verify PHP binary - proceeding anyway");
        }
    }

    // Path for the global symlink
    let link_path = Path::new("/usr/local/bin/php");
    println!("🔗 Creating symlink at: {}", link_path.display());

    // Remove existing symlink if present
    if link_path.exists() {
        println!("🗑️  Removing existing PHP symlink...");
        match std::fs::remove_file(link_path) {
            Ok(_) => println!("✅ Old symlink removed successfully"),
            Err(e) => {
                return Err(format!(
                    "❌ Failed to remove existing symlink: {}\n💡 You may need sudo privileges: sudo palawija use {}", 
                    e, version
                ).into());
            }
        }
    }
    
    // Create new symlink
    println!("🔗 Creating new symlink...");
    match symlink(&php_bin_path, &link_path) {
        Ok(_) => {
            println!("✅ Symlink created successfully!");
        }
        Err(e) => {
            return Err(format!(
                "❌ Failed to create symlink: {}\n💡 You may need sudo privileges: sudo palawija use {}", 
                e, version
            ).into());
        }
    }
    
    // Verify the switch was successful
    println!("🧪 Verifying the switch...");
    match Command::new("php").arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                let version_output = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = version_output.lines().next() {
                    println!("🎊 Success! Current PHP version: {}", first_line.trim());
                }
            }
        }
        Err(_) => {
            println!("⚠️  Could not verify the switch, but symlink was created");
        }
    }
    
    println!("\n✅ PHP version {} is now your system default! 🚀", version);
    println!("💡 Try running: php --version");
    println!("💡 Location: {}", link_path.display());
    
    Ok(())
}