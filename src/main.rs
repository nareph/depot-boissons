// src/main.rs
#![recursion_limit = "256"]

// Déclarer tous les modules de l'application
pub mod auth;
pub mod change_password;
pub mod change_password_user;
pub mod db;
pub mod error;
pub mod login;
pub mod main_window_manager;
pub mod models;
pub mod queries;
pub mod schema;
pub mod seed;
mod services;
mod config;
mod helpers;

use error::AppResult;

// Le module `ui` qui va contenir tous les composants Slint
pub mod ui {
    slint::include_modules!();
}

/// Point d'entrée principal de l'application.
fn main() -> AppResult<()> {
    // 1. Initialisation
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Application en cours de démarrage...");
    db::init()?;

    // 2. Tâches de démarrage optionnelles (comme le seeding)
    run_startup_tasks()?;

    // 3. Lancer le flux de l'application
    run_app_flow()?;

    log::info!("Application terminée.");
    Ok(())
}

/// Exécute les tâches de démarrage comme le seeding de la base de données.
fn run_startup_tasks() -> AppResult<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--seed".to_string()) {
        log::info!("Argument --seed détecté. Lancement du seeding...");
        let mut conn = db::get_conn()?;
        if let Err(e) = seed::seed_database(&mut conn) {
            log::error!("Erreur lors du seeding de la base de données : {}", e);
        }
    }
    Ok(())
}

/// Gère la boucle principale de l'application: login -> main window -> logout -> login...
fn run_app_flow() -> AppResult<()> {
    // Boucle infinie pour permettre la déconnexion/reconnexion
    loop {
        // Affiche l'écran de connexion et attend un utilisateur valide
        let user = match login::show()? {
            Some(u) => u,
            None => {
                // L'utilisateur a fermé la fenêtre de login
                log::info!("Fenêtre de connexion fermée. L'application se termine.");
                return Ok(());
            }
        };

        // Gère le changement de mot de passe obligatoire
        if user.must_change_password == 1 {
            if !change_password::show_and_update(&user)? {
                log::info!("Changement de mot de passe annulé. Retour à l'écran de connexion.");
                continue; // Retourne au début de la boucle `loop`
            }
        }

        // Lance la fenêtre principale et attend que l'utilisateur se déconnecte ou quitte
        match main_window_manager::run(&user)? {
            main_window_manager::WindowCloseReason::Logout => {
                log::info!("Déconnexion. Retour à l'écran de connexion.");
                continue; // Retourne au début de la boucle `loop`
            }
            main_window_manager::WindowCloseReason::Exit => {
                log::info!("L'utilisateur a fermé l'application depuis la fenêtre principale.");
                break; // Sort de la boucle `loop` et termine l'application
            }
        }
    }
    Ok(())
}
