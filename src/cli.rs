use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gitlet", about = "Lightweight local git contexts inside a repo")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new named gitlet
    Init {
        /// Name of the gitlet (default: "default")
        name: Option<String>,
    },
    /// Stage one or more files in a gitlet
    Add {
        /// Files to stage
        files: Vec<String>,
        /// Target gitlet (overrides active)
        #[arg(long)]
        to: Option<String>,
    },
    /// Untrack a file from a gitlet
    Remove {
        /// File to untrack
        file: String,
        /// Target gitlet (overrides active)
        #[arg(long)]
        to: Option<String>,
    },
    /// Commit staged changes in a gitlet
    Commit {
        /// Commit message
        #[arg(short)]
        m: String,
        /// Target gitlet (overrides active)
        #[arg(long)]
        to: Option<String>,
    },
    /// Show status of gitlets
    Status {
        /// Name of a specific gitlet
        name: Option<String>,
    },
    /// Show commit history of a gitlet
    Log {
        /// Name of a specific gitlet
        name: Option<String>,
    },
    /// List all gitlets in the repo
    List,
    /// Switch the active gitlet
    Switch {
        /// Name of the gitlet to activate
        name: String,
    },
}
