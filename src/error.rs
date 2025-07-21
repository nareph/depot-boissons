// src/error.rs

use bcrypt::BcryptError;
use bigdecimal::ParseBigDecimalError;
use diesel::ConnectionError;
use diesel::result::Error as DieselError;
use rust_xlsxwriter::XlsxError;
use slint::PlatformError;
use std::env::VarError;
use std::error::Error as StdError;
use std::fmt;

/// Type alias pour simplifier l'écriture des types de retour.
pub type AppResult<T> = Result<T, AppError>;

/// L'énumération principale pour toutes les erreurs possibles dans notre application.
#[derive(Debug)]
pub enum AppError {
    /// Erreur provenant de la base de données (Diesel) ou autre erreur externe "boxée".
    Database(Box<dyn StdError + Send + Sync>),

    /// Erreur provenant de l'interface graphique (Slint).
    Platform(PlatformError),

    /// Erreur spécifique au processus de "seeding".
    Seeding(String),

    /// Erreur spécifique au processus d'authentification.
    Authentication(String),

    /// Erreur d'autorisation (accès refusé).
    Unauthorized(String),

    /// Erreur de validation des données (ex: champ manquant).
    ValidationError(String),

    /// Erreur standard d'entrée/sortie (utilisée par la sauvegarde de fichiers, PDF).
    Io(std::io::Error),

    /// Erreur liée à l'impression.
    PrintingError(String),

    /// Erreur spécifique à la génération de fichiers Excel.
    ExcelGeneration(XlsxError),

    /// Erreur générique pour les messages d'erreur personnalisés.
    Generic(String),
}

/// Implémentation pour afficher l'erreur de manière lisible.
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(err) => write!(f, "Erreur de base de données : {}", err),
            AppError::Platform(err) => write!(f, "Erreur de la plateforme UI : {}", err),
            AppError::Seeding(msg) => write!(f, "Erreur de seeding : {}", msg),
            AppError::Authentication(msg) => write!(f, "Erreur d'authentification : {}", msg),
            AppError::ValidationError(e) => write!(f, "Erreur de validation: {}", e),
            AppError::Unauthorized(msg) => write!(f, "Erreur d'autorisation : {}", msg),
            AppError::Io(err) => write!(f, "Erreur d'entrée/sortie : {}", err),
            AppError::ExcelGeneration(err) => write!(f, "Erreur de génération Excel : {}", err),
            AppError::PrintingError(msg) => write!(f, "Erreur d'impression : {}", msg),
            AppError::Generic(msg) => write!(f, "Erreur : {}", msg),
        }
    }
}

/// Implémentation du trait Error pour une meilleure interopérabilité.
impl StdError for AppError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            AppError::Database(err) => Some(err.as_ref()),
            AppError::Platform(err) => Some(err),
            AppError::Io(err) => Some(err),
            AppError::ExcelGeneration(err) => Some(err),
            _ => None, // Pour les variantes basées sur String
        }
    }
}

// --- Blocs de conversion `From` pour l'opérateur `?` ---

impl From<DieselError> for AppError {
    fn from(err: DieselError) -> Self {
        AppError::Database(Box::new(err))
    }
}

impl From<ConnectionError> for AppError {
    fn from(err: ConnectionError) -> Self {
        AppError::Database(Box::new(err))
    }
}

impl From<BcryptError> for AppError {
    fn from(err: BcryptError) -> Self {
        AppError::Generic(format!("Erreur de hachage : {}", err))
    }
}

impl From<ParseBigDecimalError> for AppError {
    fn from(err: ParseBigDecimalError) -> Self {
        AppError::Generic(format!("Erreur de parsing de nombre : {}", err))
    }
}

impl From<PlatformError> for AppError {
    fn from(err: PlatformError) -> Self {
        AppError::Platform(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<VarError> for AppError {
    fn from(err: VarError) -> Self {
        AppError::Generic(format!("Variable d'environnement manquante : {}", err))
    }
}

impl From<Box<dyn StdError + Send + Sync>> for AppError {
    fn from(err: Box<dyn StdError + Send + Sync>) -> Self {
        AppError::Database(err)
    }
}

impl From<XlsxError> for AppError {
    fn from(err: XlsxError) -> Self {
        AppError::ExcelGeneration(err)
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

impl From<escpos_rs::Error> for AppError {
    fn from(err: escpos_rs::Error) -> Self {
        AppError::PrintingError(format!("Erreur ESC/POS : {}", err))
    }
}

// Ajout pour la gestion des erreurs de parsing de dates si nécessaire
impl From<chrono::ParseError> for AppError {
    fn from(err: chrono::ParseError) -> Self {
        AppError::Generic(format!("Erreur de parsing de date : {}", err))
    }
}