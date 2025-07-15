// src/main_window_manager/mod.rs

// Déclarer les sous-modules
mod dashboard_callbacks;
mod printer_callbacks;
mod product_callbacks;
mod sale_callbacks;
mod user_callbacks;

use crate::error::AppResult;
use crate::models::User;
use crate::{change_password_user, ui};
use slint::{CloseRequestResponse, ComponentHandle};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowCloseReason {
    Logout,
    Exit,
}

pub fn run(user: &User) -> AppResult<WindowCloseReason> {
    let main_window = ui::MainWindow::new()?;
    main_window.window().set_maximized(true);
    main_window.set_is_admin(user.role == "Admin");
    main_window.set_welcome_message(user.name.clone().into());

    // --- Gestion de la fermeture de la fenêtre ---
    let logout_requested = std::rc::Rc::new(std::cell::RefCell::new(false));
    let logout_requested_clone = logout_requested.clone();

    main_window.window().on_close_requested(move || {
        let is_logout = *logout_requested_clone.borrow();
        if is_logout {
            log::info!("Fermeture de fenêtre suite à une déconnexion.");
        } else {
            log::info!("Fermeture de l'application demandée par l'utilisateur.");
        }
        CloseRequestResponse::HideWindow
    });

    setup_callbacks(&main_window, user, logout_requested.clone());

    log::info!("Déclenchement du chargement initial des données...");
    main_window.invoke_request_dashboard_data();
    main_window.invoke_request_products();
    if user.role == "Admin" {
        main_window.invoke_request_users();
    }

    main_window.run()?;

    let close_reason = if *logout_requested.borrow() {
        WindowCloseReason::Logout
    } else {
        WindowCloseReason::Exit
    };
    Ok(close_reason)
}

fn setup_callbacks(
    main_window: &ui::MainWindow,
    user: &User,
    logout_flag: std::rc::Rc<std::cell::RefCell<bool>>,
) {
    let main_window_handle = main_window.as_weak();
    let current_user_id = user.id;

    // Déconnexion
    let logout_handle = main_window_handle.clone();
    main_window.on_logout_clicked(move || {
        if let Some(ui) = logout_handle.upgrade() {
            log::info!("Déconnexion demandée.");
            *logout_flag.borrow_mut() = true;
            let _ = ui.hide();
        }
    });

    // Changement de mot de passe
    main_window.on_change_password_clicked(move || {
        if let Err(e) = change_password_user::show(current_user_id) {
            log::error!(
                "Erreur lors de l'ouverture de la fenêtre de changement de mot de passe: {}",
                e
            );
        }
    });

    // Déléguer aux modules spécialisés
    dashboard_callbacks::setup(&main_window_handle);
    product_callbacks::setup(&main_window_handle);
    sale_callbacks::setup(&main_window.as_weak(), user.id, user.role == "Admin");

    // Configuration des callbacks pour les imprimantes
    printer_callbacks::setup(&main_window_handle);

    // Charger les imprimantes au démarrage
    main_window.invoke_load_printers();

    if user.role == "Admin" {
        user_callbacks::setup(&main_window_handle, user.id);
    }
}

/// Affiche un dialogue d'information standard.
pub fn show_info_dialog(title: &str, message: &str) {
    if let Ok(dialog) = ui::InfoDialog::new() {
        dialog.set_dialog_title(title.into());
        dialog.set_message(message.into());
        let handle = dialog.as_weak();
        dialog.on_ok_clicked(move || {
            if let Some(d) = handle.upgrade() {
                let _ = d.hide();
            }
        });
        let _ = dialog.run();
    }
}

/// Affiche un dialogue d'erreur standard.
pub fn show_error_dialog(title: &str, message: &str) {
    if let Ok(dialog) = ui::ErrorDialog::new() {
        dialog.set_dialog_title(title.into());
        dialog.set_message(message.into());
        let handle = dialog.as_weak();
        dialog.on_ok_clicked(move || {
            if let Some(d) = handle.upgrade() {
                let _ = d.hide();
            }
        });
        let _ = dialog.run();
    }
}
