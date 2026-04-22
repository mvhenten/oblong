mod gui;
mod snap;
mod config;
mod outputs;
mod appearance;
mod defaults;
mod switcher;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "oblong", about = "Window management for Sway")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Snap the focused window in a direction (cycles half → third → two-thirds)
    Snap {
        /// Direction: left, right, up, down, maximize, center, restore
        direction: String,
    },
    /// Switch focus to another window (macOS-style Cmd+Tab)
    Switch {
        /// Direction: next, prev
        direction: String,
    },
    /// Open the GUI shortcut editor
    Gui,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Snap { direction }) => {
            if let Err(e) = snap::snap(&direction) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Some(Commands::Switch { direction }) => {
            if let Err(e) = switcher::switch(&direction) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        Some(Commands::Gui) | None => {
            gui::run().expect("Failed to run GUI");
        }
    }
}
