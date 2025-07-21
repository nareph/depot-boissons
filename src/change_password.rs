// src/change_password.rs

use crate::error::AppResult;
use crate::models::User;
use crate::queries;
use crate::ui::ChangePasswordWindow;
use slint::ComponentHandle;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn show_and_update(user: &User) -> AppResult<bool> {
    log::info!(
        "Affichage de la fenêtre de changement de mot de passe pour l'utilisateur '{}'",
        user.name
    );

    let ui = ChangePasswordWindow::new()?;
    let password_changed = Arc::new(Mutex::new(false));
    let password_changed_clone = password_changed.clone();
    let ui_handle = ui.as_weak();
    let user_id = user.id.clone(); // Clone the user ID for use in the thread

    ui.on_confirm_clicked(move |new_password| {
        if let Some(ui) = ui_handle.upgrade() {
            let password_changed_clone_inner = password_changed_clone.clone();

            ui.set_loading(true);
            ui.set_status_text("Mise à jour en cours...".into());

            let thread_ui_handle = ui_handle.clone();
            let user_id_for_thread = user_id.clone(); // Clone the Arc, not the String


            thread::spawn(move || {
                let hash_result = bcrypt::hash(&new_password, bcrypt::DEFAULT_COST);

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = thread_ui_handle.upgrade() {
                        match hash_result {
                            Ok(hashed_password) => {
                                match queries::update_user_password(&user_id_for_thread, &hashed_password) {
                                    Ok(_) => {
                                        log::info!("Mot de passe mis à jour avec succès pour l'utilisateur ID {}", user_id_for_thread);
                                        *password_changed_clone_inner.lock().unwrap() = true;
                                        let _ = ui.hide();
                                    }
                                    Err(e) => {
                                        log::error!("Erreur BDD lors de la mise à jour du mot de passe: {}", e);
                                        ui.set_status_text("Erreur".into());
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Erreur de hachage du mot de passe: {}", e);
                                ui.set_status_text("Erreur".into());
                            }
                        }
                        ui.set_loading(false);
                    }
                });
            });
        }
    });

    ui.run()?;

    if let Ok(guard) = password_changed.try_lock() {
        let result = *guard;
        if result {
            log::info!("Fenêtre de changement de mot de passe fermée, mot de passe changé.");
        } else {
            log::info!("Fenêtre de changement de mot de passe fermée sans modification.");
        }
        Ok(result)
    } else {
        log::error!(
            "Impossible de verrouiller le Mutex après la fermeture de la fenêtre de changement de mot de passe."
        );
        Ok(false)
    }
}
