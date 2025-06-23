use diesel::result::Error as DieselError;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Database(Box<dyn std::error::Error + Send + Sync>),
    Platform(slint::PlatformError),
    Seeding(String),
    Authentication(String),
    Io(std::io::Error),
    Generic(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(err) => write!(f, "Database error: {}", err),
            AppError::Platform(err) => write!(f, "Platform error: {}", err),
            AppError::Seeding(msg) => write!(f, "Seeding error: {}", msg),
            AppError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            AppError::Io(err) => write!(f, "IO error: {}", err),
            AppError::Generic(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Database(err) => Some(err.as_ref()),
            AppError::Platform(err) => Some(err),
            AppError::Io(err) => Some(err),
            AppError::Seeding(_) | AppError::Authentication(_) | AppError::Generic(_) => None,
        }
    }
}

// Convert from different error types
impl From<Box<dyn std::error::Error + Send + Sync>> for AppError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        AppError::Database(err)
    }
}

impl From<slint::PlatformError> for AppError {
    fn from(err: slint::PlatformError) -> Self {
        AppError::Platform(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Generic(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::Generic(msg.to_string())
    }
}

impl From<DieselError> for AppError {
    fn from(err: DieselError) -> Self {
        // On encapsule l'erreur Diesel dans notre variante AppError::Database
        // C'est le comportement le plus logique pour une erreur venant de cette crate.
        AppError::Database(Box::new(err))
    }
}

// Helper type alias for Results
pub type AppResult<T> = Result<T, AppError>;
