use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitletError {
    #[error("Not inside a git repository")]
    NotInGitRepo,

    #[error("No gitlets found. Run 'gitlet init' first.")]
    NoGitletsFound,

    #[error("gitlet '{0}' does not exist. Run 'gitlet list' to see all gitlets.")]
    GitletNotFound(String),

    #[error("'{0}' is not tracked by gitlet '{1}'")]
    FileNotTracked(String, String),

    #[error("'{0}' is already tracked by gitlet '{1}'")]
    FileAlreadyTracked(String, String),

    #[error("'{0}' does not exist")]
    FileNotFound(String),

    #[error("'{0}' is outside the git repository")]
    FileOutsideRepo(String),

    #[error("gitlet '{0}' already exists. Run 'gitlet list' to see all gitlets.")]
    GitletAlreadyExists(String),
}
