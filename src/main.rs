use clap::{ Parser, Subcommand };
use std::process::Command;
use std::io::{self, Write};
use std::env;
use std::path::{Path, PathBuf};

// For symbolic links on Linux/macOS
#[cfg(target_family = "unix")]
use std::os::unix::fs::symlink;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "A PHP version manager inspired by Agritaloka.",
    long_about = None
)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Installs a specific version of PHP
    Install {
        /// The PHP version to install (e.g., 8.1.10)
        version: String,
    },
    /// Sets the global PHP version to use
    Use {
        /// The PHP version to use
        version: String,
    },
    /// Lists all installed PHP versions
    List,
    /// Displays the path to the currently active PHP binary
    Which
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
            // TODO: Add logic to list versions here
        }
        Commands::Which => {
            if cfg!(target_os = "windows") {
                let output = Command::new("where.exe").arg("php").output().expect("Failed to execute command");
                println!("{}", String::from_utf8_lossy(&output.stdout));
            } else {
                let output = Command::new("which").arg("php").output().expect("Failed to execute command");
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
        }
    }
}

// Function to install PHP
fn install_php(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("⚙️  Installing PHP version: {}", version);

    // Determine the installation directory
    let install_dir = if cfg!(target_os = "windows") {
        let app_data = env::var("APPDATA")?;
        format!("{}\\palawija", app_data)
    } else {
        format!("{}/.palawija", env::var("HOME")?)
    };

    // Create the directory if it doesn't exist
    std::fs::create_dir_all(&install_dir)?;

    // Determine the download URL
    // This is an example, the actual URL may need to be adjusted
    let php_url = format!("https://www.php.net/distributions/php-{}.tar.gz", version);

    if cfg!(target_os = "windows") {
        // Installation logic for Windows
        println!("⬇️ Downloading PHP from {}...", php_url);
        let download_path = format!("{}\\php-{}.zip", install_dir, version);
        
        // Use Powershell to download and extract
        let output = Command::new("powershell.exe")
            .arg("-Command")
            .arg(format!("Invoke-WebRequest -Uri {} -OutFile {}; Expand-Archive -Path {} -DestinationPath {}", php_url, download_path, download_path, install_dir))
            .output()?;
        
        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;

        if !output.status.success() {
            return Err("Failed to install PHP on Windows".into());
        }

    } else {
        // Installation logic for macOS and Linux
        println!("⬇️ Downloading PHP from {}...", php_url);
        let tar_gz_path = format!("{}/php-{}.tar.gz", install_dir, version);

        // Download the file using `curl` or `wget`
        Command::new("curl")
            .arg("-L")
            .arg(&php_url)
            .arg("-o")
            .arg(&tar_gz_path)
            .status()?;
        
        // Extract the file
        println!("📂 Extracting files to {}...", install_dir);
        let extracted_dir = format!("{}/php-{}", install_dir, version);
        std::fs::create_dir(&extracted_dir)?;

        Command::new("tar")
            .arg("-xzf")
            .arg(&tar_gz_path)
            .arg("-C")
            .arg(&extracted_dir)
            .status()?;
        
        // Optional: delete the tar.gz file after extraction
        std::fs::remove_file(&tar_gz_path)?;

        // TODO: You may need to add compilation steps here if downloading from source code
        // for example: `./configure`, `make`, `make install`
        // This would be much more complex and depends on system dependencies
        println!("✅ PHP installed to {}", extracted_dir);
    }

    println!("🎉 PHP version {} installed successfully! 🎉", version);
    Ok(())
}

fn use_php(version: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Setting default PHP version to: {}", version);

    let install_dir = if cfg!(target_os = "windows") {
        let app_data = env::var("APPDATA")?;
        PathBuf::from(format!("{}\\palawija", app_data))
    } else {
        let home = env::var("HOME")?;
        PathBuf::from(format!("{}/.palawija", home))
    };

    let php_bin_path = if cfg!(target_os = "windows") {
        // Location of php.exe
        install_dir.join(format!("php-{}", version)).join("php.exe")
    } else {
        // Location of the php binary
        install_dir.join(format!("php-{}", version)).join("bin").join("php")
    };

    if !php_bin_path.exists() {
        return Err(format!("PHP version {} is not installed at {:?}", version, php_bin_path).into());
    }

    if cfg!(target_os = "windows") {
        // Logic for Windows
        // Use Powershell to modify the 'Path' environment variable
        let current_path = env::var("PATH").unwrap_or_default();
        let new_path = format!("{};{}", php_bin_path.parent().unwrap().to_str().unwrap(), current_path);

        let output = Command::new("powershell.exe")
            .arg("-Command")
            .arg(format!("[Environment]::SetEnvironmentVariable('Path', '{}', 'User')", new_path))
            .output()?;
            
        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;

        if !output.status.success() {
            return Err("Failed to set Path environment variable on Windows. Try running the program as an administrator.".into());
        }

    } else {
        // Logic for Linux/macOS (using symlink)
        let link_path = Path::new("/usr/local/bin/php");
        if link_path.exists() {
            std::fs::remove_file(link_path)?;
        }
        symlink(&php_bin_path, &link_path)?;
    }
    
    println!("✅ PHP version {} has been successfully set as default! ✅", version);
    Ok(())
}