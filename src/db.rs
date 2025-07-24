// src/db.rs
use crate::error::AppResult;
use bcrypt::{DEFAULT_COST, hash};
use diesel::{Connection, sqlite::SqliteConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use dotenvy::dotenv;
use std::env;
use uuid::Uuid;

pub const MIGRATIONS: EmbeddedMigrations =
    embed_migrations!("migrations/2025-06-22-213540_create_initial_tables");

/// Initialise la base de données et exécute les migrations
pub fn init() -> AppResult<()> {
    log::info!("Initialisation de la base de données SQLite...");
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;

    let mut conn = SqliteConnection::establish(&database_url)?;

    log::info!("Exécution des migrations SQLite...");
    conn.run_pending_migrations(MIGRATIONS)?;

    // Vérification/création de l'utilisateur admin après les migrations
    ensure_admin_user(&mut conn)?;

    log::info!("Migrations et vérifications terminées.");
    Ok(())
}

/// Obtient une nouvelle connexion à la base de données
pub fn get_conn() -> AppResult<SqliteConnection> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;
    Ok(SqliteConnection::establish(&database_url)?)
}

/// Garantit l'existence d'un utilisateur admin avec mot de passe sécurisé
fn ensure_admin_user(conn: &mut SqliteConnection) -> AppResult<()> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;

    let admin_exists = users
        .filter(name.eq("admin"))
        .first::<crate::models::User>(conn)
        .optional()?;

    if admin_exists.is_none() {
        let admin_id = Uuid::new_v4().to_string();
        let password_hash = hash_default_admin_password()?;

        diesel::insert_into(users)
            .values((
                id.eq(admin_id),
                name.eq("admin"),
                password.eq(password_hash),
                role.eq("Admin"),
                must_change_password.eq(1), // Force le changement au premier login
            ))
            .execute(conn)?;

        log::warn!("⚠️ Utilisateur admin créé. Changez le mot de passe immédiatement !");
    }

    Ok(())
}

/// Génère un hash bcrypt sécurisé pour le mot de passe admin par défaut
fn hash_default_admin_password() -> AppResult<String> {
    // En production, utilisez une valeur aléatoire complexe plutôt que "admin"
    Ok(hash("admin", DEFAULT_COST)?)
}

/// Fonction utilitaire pour les tests - NE PAS UTILISER EN PRODUCTION
#[cfg(test)]
pub fn create_test_admin(conn: &mut SqliteConnection) -> AppResult<()> {
    use crate::schema::users::dsl::*;

    let test_hash = hash("testadmin", DEFAULT_COST)?;

    diesel::insert_or_ignore_into(users)
        .values((
            id.eq(Uuid::new_v4().to_string()),
            name.eq("testadmin"),
            password.eq(test_hash),
            role.eq("Admin"),
            must_change_password.eq(0),
        ))
        .execute(conn)?;

    Ok(())
}
