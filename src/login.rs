// src/login.rs

use crate::auth;
use crate::error::AppResult;
use crate::models::User;
use crate::ui::LoginWindow;
use slint::ComponentHandle;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn show() -> AppResult<Option<User>> {
    log::info!("Affichage de la fenêtre de connexion.");
    let ui = LoginWindow::new()?;

    let authenticated_user = Arc::new(Mutex::new(None::<User>));

    let authenticated_user_clone = authenticated_user.clone();
    let ui_handle = ui.as_weak();

    ui.on_login_clicked(move |login, password| {
        if let Some(ui) = ui_handle.upgrade() {
            log::info!("Tentative de connexion pour l'utilisateur : {}", login);

            let auth_ui_handle = ui.as_weak();
            let auth_user_clone = authenticated_user_clone.clone();

            ui.set_loading(true);
            ui.set_status_text("Connexion en cours...".into());

            thread::spawn(move || {
                let auth_result = auth::authenticate(&login, &password);

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = auth_ui_handle.upgrade() {
                        ui.set_loading(false);
                        match auth_result {
                            Ok(user) => {
                                log::info!(
                                    "Authentification réussie pour l'utilisateur ID: {}",
                                    user.id
                                );
                                ui.set_status_text("Connexion réussie !".into());

                                *auth_user_clone.lock().unwrap() = Some(user);

                                let _ = ui.hide();
                            }
                            Err(error_message) => {
                                log::warn!(
                                    "Échec de l'authentification pour '{}': {}",
                                    login,
                                    error_message
                                );
                                ui.set_status_text(error_message.to_string().into());
                                ui.set_password_text("".into());
                            }
                        }
                    }
                });
            });
        }
    });

    ui.run()?;

    if let Ok(mut guard) = authenticated_user.try_lock() {
        // .take() déplace la valeur hors de l'Option, la remplaçant par None.
        let result = guard.take();
        if result.is_some() {
            log::info!("Fenêtre de connexion fermée avec un utilisateur authentifié.");
        } else {
            log::info!("Fenêtre de connexion fermée sans authentification.");
        }
        Ok(result)
    } else {
        // Ce cas est très peu probable mais gère le cas où on ne peut pas obtenir le verrou.
        log::error!(
            "Impossible de verrouiller le Mutex de l'utilisateur après la fermeture de la fenêtre."
        );
        Ok(None)
    }
}
