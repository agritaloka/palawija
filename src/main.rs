use clap::{ Parser, Subcommand };

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

use std::process::Command;

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Install { version } => {
            println!("Installing PHP version: {}", version);
            // TODO: Add installation logic here
        }
        Commands::Use { version } => {
            println!("Using PHP version: {}", version);
            // TODO: Add logic to switch versions here
        }
        Commands::List => {
            println!("List of installed PHP versions:");
            // TODO: Add logic to list versions here
        }
        Commands::Which => {
            if cfg!(target_os = "windows") {
                // For Windows
                let output = Command::new("where.exe").arg("php").output().expect("Failed to execute command");
                println!("{}", String::from_utf8_lossy(&output.stdout));
            } else {
                // For macOS and Linux
                let output = Command::new("which").arg("php").output().expect("Failed to execute command");
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
        }
    }
}