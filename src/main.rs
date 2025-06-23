use slint::ComponentHandle;

// Déclarer les modules de logique
pub mod auth;
pub mod change_password;
pub mod db;
pub mod error;
pub mod login;
pub mod models;
mod queries;
pub mod schema;
pub mod seed;

// Import our custom error types
use error::{AppError, AppResult};

// C'EST LA LIGNE LA PLUS IMPORTANTE
// On crée un module `ui` qui contiendra TOUS les composants exportés
// de TOUS les fichiers .slint compilés par build.rs.
pub mod ui {
    slint::include_modules!();
}

fn main() -> AppResult<()> {
    env_logger::init();
    log::info!("Application en cours de démarrage...");

    // Now db::init() error is automatically converted
    db::init()?;

    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--seed".to_string()) {
        log::info!("Argument --seed détecté. Lancement du seeding...");
        let mut conn = db::get_conn()?;
        if let Err(e) = seed::seed_database(&mut conn) {
            log::error!("Erreur lors du seeding de la base de données : {}", e);
            return Err(AppError::Seeding(e.to_string()));
        } else {
            log::info!("Base de données seedée avec succès.");
        }
    }

    match login::show() {
        Ok(Some(mut user)) => {
            log::info!(
                "L'utilisateur '{}' s'est connecté. Lancement de l'application principale.",
                user.name
            );

            if user.must_change_password {
                log::info!(
                    "L'utilisateur '{}' doit changer son mot de passe.",
                    user.name
                );

                match change_password::show_and_update(&user) {
                    Ok(true) => {
                        log::info!("Mot de passe changé. L'utilisateur peut continuer.");
                        // L'objet `user` en mémoire est maintenant "périmé" (le flag est toujours à true),
                        // mais ce n'est pas grave car la base de données est à jour.
                        // Pour la session actuelle, on peut le mettre à jour manuellement.
                        user.must_change_password = false;
                    }
                    Ok(false) => {
                        log::warn!(
                            "L'utilisateur a fermé la fenêtre sans changer son mot de passe. Déconnexion."
                        );
                        return Ok(()); // On termine l'application
                    }
                    Err(e) => {
                        log::error!(
                            "Erreur lors du processus de changement de mot de passe: {}",
                            e
                        );
                        return Err(e);
                    }
                }
            }

            // Slint errors are automatically converted
            let main_window = ui::MainWindow::new()?;
            main_window.set_welcome_message(
                format!("Bienvenue, {}. Votre rôle : {}", user.name, user.role).into(),
            );
            main_window.run()?;
        }
        Ok(None) => {
            log::info!("L'application a été fermée par l'utilisateur à l'écran de connexion.");
        }
        Err(e) => {
            log::error!("Erreur fatale de l'interface graphique : {}", e);
            return Err(e.into());
            //return Err(AppError::Platform(e));
        }
    }

    log::info!("Application terminée.");
    Ok(())
}
