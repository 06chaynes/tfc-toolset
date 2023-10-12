use miette::Diagnostic;
use thiserror::Error;

/// A generic “error” type
#[derive(Error, Diagnostic, Debug)]
pub enum CleanError {
    /// Git related errors
    #[error(transparent)]
    #[diagnostic(
        code(clean_workspace::git),
        help("My bad, something went wrong with git!")
    )]
    Git(#[from] git2::Error),
    /// Walkdir related errors
    #[error(transparent)]
    #[diagnostic(
        code(clean_workspace::walkdir),
        help("Oh Bother, something went walking the directory!")
    )]
    Walkdir(#[from] walkdir::Error),
}
