// src/db.rs

use crate::error::AppResult;
use diesel::{Connection, sqlite::SqliteConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use dotenvy::dotenv;
use std::env;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub fn init() -> AppResult<()> {
    log::info!("Initialisation de la base de données SQLite...");
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;

    let mut conn = SqliteConnection::establish(&database_url)?;

    log::info!("Exécution des migrations SQLite...");
    conn.run_pending_migrations(MIGRATIONS)?;
    log::info!("Migrations terminées.");

    Ok(())
}

pub fn get_conn() -> AppResult<SqliteConnection> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;
    Ok(SqliteConnection::establish(&database_url)?)
}
