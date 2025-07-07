// src/error.rs

use bcrypt::BcryptError;
use bigdecimal::ParseBigDecimalError;
use diesel::ConnectionError;
use diesel::result::Error as DieselError;
use slint::PlatformError;
use std::error::Error as StdError;
use std::fmt;

/// Type alias pour simplifier l'écriture des types de retour.
pub type AppResult<T> = Result<T, AppError>;

/// L'énumération principale pour toutes les erreurs possibles dans notre application.
#[derive(Debug)]
pub enum AppError {
    /// Erreur provenant de la base de données (Diesel).
    /// On utilise `Box<dyn StdError ...>` pour pouvoir y stocker `DieselError` et `ConnectionError`.
    Database(Box<dyn StdError + Send + Sync>),

    /// Erreur provenant de l'interface graphique (Slint).
    Platform(PlatformError),

    /// Erreur spécifique au processus de "seeding" (remplissage) de la base de données.
    Seeding(String),

    /// Erreur spécifique au processus d'authentification (ex: mauvais mot de passe).
    Authentication(String),

    /// Erreur d'autorisation (ex: accès refusé à une ressource).
    Unauthorized(String),

    ///
    ValidationError(String),

    /// Erreur standard d'entrée/sortie.
    Io(std::io::Error),

    /// Erreur générique pour les messages d'erreur personnalisés créés dans notre code.
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
            AppError::Seeding(_)
            | AppError::Authentication(_)
            | AppError::Unauthorized(_)
            | AppError::Generic(_) => None,
            AppError::ValidationError(_) => None,
        }
    }
}

// --- Blocs de conversion `From` pour l'opérateur `?` ---

// Convertit les erreurs de requête Diesel en AppError::Database.
impl From<DieselError> for AppError {
    fn from(err: DieselError) -> Self {
        AppError::Database(Box::new(err))
    }
}

// Convertit les erreurs de connexion Diesel en AppError::Database.
impl From<ConnectionError> for AppError {
    fn from(err: ConnectionError) -> Self {
        AppError::Database(Box::new(err))
    }
}

// Convertit les erreurs de Bcrypt en AppError::Generic (ou une nouvelle variante si vous préférez).
impl From<BcryptError> for AppError {
    fn from(err: BcryptError) -> Self {
        AppError::Generic(format!("Erreur de hachage : {}", err))
    }
}

// Convertit les erreurs de BigDecimal en AppError::Generic.
impl From<ParseBigDecimalError> for AppError {
    fn from(err: ParseBigDecimalError) -> Self {
        AppError::Generic(format!("Erreur de parsing de nombre : {}", err))
    }
}

// Les autres conversions que vous aviez déjà.
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

// Ajout pour la compatibilité avec env::var
impl From<std::env::VarError> for AppError {
    fn from(err: std::env::VarError) -> Self {
        AppError::Generic(format!(
            "Variable d'environnement manquante ou invalide : {}",
            err
        ))
    }
}

impl From<Box<dyn StdError + Send + Sync>> for AppError {
    fn from(err: Box<dyn StdError + Send + Sync>) -> Self {
        AppError::Database(err)
    }
}
