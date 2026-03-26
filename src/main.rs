mod cli;
mod config;
mod exclude;
mod gitlet;
mod repo;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => {
            let git_root = repo::find_git_root()?;
            let name = name.as_deref().unwrap_or("default");
            gitlet::init(&git_root, name)
        }
        Commands::Add { files, to } => {
            let git_root = repo::find_git_root()?;
            gitlet::add(&git_root, &files, to.as_deref())
        }
        Commands::Remove { file, to } => {
            let git_root = repo::find_git_root()?;
            gitlet::remove(&git_root, &file, to.as_deref())
        }
        Commands::Commit { m, to } => {
            let git_root = repo::find_git_root()?;
            gitlet::commit(&git_root, &m, to.as_deref())
        }
        Commands::Status { .. } => {
            println!("[status] not yet implemented");
            Ok(())
        }
        Commands::Log { .. } => {
            println!("[log] not yet implemented");
            Ok(())
        }
        Commands::List => {
            println!("[list] not yet implemented");
            Ok(())
        }
        Commands::Switch { .. } => {
            println!("[switch] not yet implemented");
            Ok(())
        }
    }
}
