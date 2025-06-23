// src/db.rs

use crate::error::{AppError, AppResult};
use diesel::{pg::PgConnection, prelude::*};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use dotenvy::dotenv;
use std::env;

pub const MIGRATIONS: EmbeddedMigrations =
    embed_migrations!("migrations/2025-06-22-213540_create_initial_tables"); // Assurez-vous que ce chemin est correct

/// Initialise la base de données et exécute les migrations en attente.
///
/// Cette fonction retourne maintenant notre type `AppResult` personnalisé pour une gestion
/// d'erreurs centralisée.
pub fn init() -> AppResult<()> {
    log::info!("Initialisation de la base de données et exécution des migrations...");

    // On gère l'erreur si le fichier .env n'est pas trouvé
    dotenv()
        .map_err(|e| AppError::Generic(format!("Impossible de charger le fichier .env: {}", e)))?;

    // On gère l'erreur si la variable d'environnement n'est pas définie
    let database_url = env::var("DATABASE_URL").map_err(|_| {
        AppError::Generic("La variable d'environnement DATABASE_URL doit être définie.".to_string())
    })?;

    // On gère l'erreur de connexion à la base de données
    let mut conn =
        PgConnection::establish(&database_url).map_err(|e| AppError::Database(Box::new(e)))?;

    log::info!("Exécution des migrations en attente...");
    // La conversion d'erreur est gérée automatiquement par `?` grâce à votre `impl From`
    conn.run_pending_migrations(MIGRATIONS)?;
    log::info!("Migrations de la base de données terminées avec succès.");

    Ok(())
}

/// Établit et retourne une nouvelle connexion à la base de données.
///
/// Au lieu de `panic!`, cette fonction retourne maintenant un `AppResult`,
/// forçant l'appelant à gérer les erreurs de connexion de manière propre.
pub fn get_conn() -> AppResult<PgConnection> {
    // On peut ne pas recharger dotenvy à chaque appel si on est sûr
    // qu'il a été appelé au démarrage, mais c'est plus sûr de le laisser.
    dotenv()
        .map_err(|e| AppError::Generic(format!("Impossible de charger le fichier .env: {}", e)))?;

    let database_url = env::var("DATABASE_URL").map_err(|_| {
        AppError::Generic("La variable d'environnement DATABASE_URL doit être définie.".to_string())
    })?;

    // Plus de `panic!` ici. On retourne une erreur `AppError::Database`.
    PgConnection::establish(&database_url).map_err(|e| AppError::Database(Box::new(e)))
}
