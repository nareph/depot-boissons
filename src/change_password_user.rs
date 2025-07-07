// src/change_password_user.rs

use crate::{error::AppResult, queries, ui::ChangePasswordUserWindow};
use slint::ComponentHandle;
use std::sync::{Arc, Mutex};
use std::thread;
use uuid::Uuid;

pub fn show(user_id: Uuid) -> AppResult<bool> {
    log::info!(
        "Affichage de la fenêtre de changement de mot de passe pour l'utilisateur ID: {}",
        user_id
    );
    let ui = ChangePasswordUserWindow::new()?;

    let password_changed = Arc::new(Mutex::new(false));
    let password_changed_clone = password_changed.clone();

    let ui_handle = ui.as_weak();

    ui.on_save_clicked(move |old_password, new_password| {
        if let Some(ui) = ui_handle.upgrade() {
            ui.set_status_text("Vérification...".into());
            let password_changed_clone_inner = password_changed_clone.clone();

            let thread_ui_handle = ui_handle.clone();

            thread::spawn(move || {
                // Vérification de l'ancien mot de passe
                let user_check_result = match queries::get_user_by_id(user_id) {
                    Ok(user) => match bcrypt::verify(&old_password, &user.password) {
                        Ok(true) => Ok(()), // L'ancien mot de passe est correct
                        Ok(false) => Err("Ancien mot de passe incorrect.".to_string()),
                        Err(e) => {
                            log::error!("Erreur de vérification bcrypt: {}", e);
                            Err("Erreur interne de vérification.".to_string())
                        }
                    },
                    Err(e) => {
                        log::error!("Impossible de trouver l'utilisateur {} pour vérifier le mot de passe: {}", user_id, e);
                        Err("Impossible de trouver l'utilisateur.".to_string())
                    },
                };

                // Si la vérification a réussi, on procède au hachage et à la mise à jour
                let final_result: Result<(), String> = match user_check_result {
                    Ok(()) => match bcrypt::hash(&new_password, bcrypt::DEFAULT_COST) {
                        Ok(hash) => {
                            log::info!("Hachage du nouveau mot de passe réussi.");
                            queries::update_user_password(user_id, &hash)
                                .map_err(|e| {
                                    log::error!("Erreur BDD lors de la mise à jour du mot de passe: {}", e);
                                    "Erreur de base de données.".to_string()
                                })
                        }
                        Err(e) => {
                            log::error!("Erreur de hachage du nouveau mot de passe: {}", e);
                            Err("Erreur interne de hachage.".to_string())
                        }
                    },
                    Err(e) => Err(e), // On propage l'erreur de la vérification initiale
                };

                // On met à jour l'UI avec le résultat final
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = thread_ui_handle.upgrade() {
                        match final_result {
                            Ok(()) => {
                                log::info!("Mot de passe changé avec succès par l'utilisateur ID {}.", user_id);
                                *password_changed_clone_inner.lock().unwrap() = true;
                                let _ = ui.hide();
                            }
                            Err(e) => {
                                ui.set_status_text(e.into());
                            }
                        }
                    }
                });
            });
        }
    });

    ui.run()?;

    Ok(*password_changed.lock().unwrap())
}
