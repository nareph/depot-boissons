// src/main_window_manager/user_callbacks.rs

use crate::{queries, ui};
use slint::{ComponentHandle, Model, Weak};
use uuid::Uuid;

/// Configure tous les callbacks liés à la gestion des utilisateurs pour la fenêtre principale.
/// Cette fonction n'est appelée que si l'utilisateur connecté est un administrateur.
pub fn setup(main_window_handle: &Weak<ui::MainWindow>, current_user_id: Uuid) {
    // --- CHARGEMENT DE LA LISTE DES UTILISATEURS ---
    let users_handle = main_window_handle.clone();
    main_window_handle
        .upgrade()
        .unwrap()
        .on_request_users(move || {
            if let Some(ui) = users_handle.upgrade() {
                log::info!("Chargement de la liste des utilisateurs...");
                match queries::get_all_users(current_user_id) {
                    Ok(data) => {
                        let model = data
                            .into_iter()
                            .map(|u| ui::UserUI {
                                id: u.id.to_string().into(),
                                name: u.name.into(),
                                role: u.role.into(),
                            })
                            .collect::<Vec<_>>();
                        ui.set_users_model(std::rc::Rc::new(slint::VecModel::from(model)).into());
                    }
                    Err(e) => log::error!("Erreur lors du chargement des utilisateurs: {}", e),
                }
            }
        });

    // --- AJOUT D'UN NOUVEL UTILISATEUR ---
    let add_handle = main_window_handle.clone();
    main_window_handle
        .upgrade()
        .unwrap()
        .on_add_user_clicked(move || {
            if let Some(main_ui) = add_handle.upgrade() {
                if let Ok(dialog) = ui::AddUserDialog::new() {
                    let main_ui_handle = main_ui.as_weak();
                    let dialog_handle = dialog.as_weak();
                    dialog.on_save_clicked(move |name, password, role| {
                        if let Some(d) = dialog_handle.upgrade() {
                            if password.trim().is_empty() {
                                d.set_status_message(
                                    "Le mot de passe ne peut pas être vide.".into(),
                                );
                                return;
                            }

                            if let Ok(hash) = bcrypt::hash(&password, bcrypt::DEFAULT_COST) {
                                if queries::create_user(&name, &hash, &role).is_ok() {
                                    log::info!("Utilisateur '{}' créé avec succès.", name);
                                    if let Some(ui) = main_ui_handle.upgrade() {
                                        ui.invoke_request_users();
                                    }
                                    let _ = d.hide();
                                } else {
                                    d.set_status_message(
                                        "Erreur : Le nom d'utilisateur existe déjà?".into(),
                                    );
                                }
                            }
                        }
                    });
                    let dialog_handle_cancel = dialog.as_weak();
                    dialog.on_cancelled(move || {
                        if let Some(d) = dialog_handle_cancel.upgrade() {
                            let _ = d.hide();
                        }
                    });
                    let _ = dialog.run();
                }
            }
        });

    // --- ÉDITION D'UN UTILISATEUR EXISTANT ---
    let edit_handle = main_window_handle.clone();
    main_window_handle
        .upgrade()
        .unwrap()
        .on_edit_user_clicked(move |user_id_str| {
            if let Some(main_ui) = edit_handle.upgrade() {
                if let Ok(user_id) = Uuid::parse_str(&user_id_str) {
                    match queries::get_user_by_id(user_id) {
                        Ok(user_to_edit) => {
                            if let Ok(dialog) = ui::EditUserDialog::new() {
                                // Pré-remplir la boîte de dialogue avec les infos de l'utilisateur
                                dialog.set_user_id(user_to_edit.id.to_string().into());
                                dialog.set_username(user_to_edit.name.into());

                                let roles = dialog.get_roles(); // Obtenir le VecModel des rôles
                                if let Some(index) =
                                    roles.iter().position(|r| r == user_to_edit.role.as_str())
                                {
                                    dialog.set_selected_role_index(index as i32);
                                }

                                let main_ui_handle = main_ui.as_weak();
                                let dialog_handle = dialog.as_weak();
                                dialog.on_save_clicked(move |id, new_name, new_role| {
                                    if let Some(d) = dialog_handle.upgrade() {
                                        if let Ok(id_uuid) = Uuid::parse_str(&id) {
                                            match queries::update_user_info(
                                                id_uuid, &new_name, &new_role,
                                            ) {
                                                Ok(_) => {
                                                    log::info!(
                                                        "Utilisateur '{}' mis à jour.",
                                                        new_name
                                                    );
                                                    if let Some(ui) = main_ui_handle.upgrade() {
                                                        ui.invoke_request_users();
                                                    }
                                                    let _ = d.hide();
                                                }
                                                Err(e) => {
                                                    log::error!(
                                                        "Erreur de mise à jour utilisateur : {}",
                                                        e
                                                    );
                                                    d.set_status_message(
                                                        "Erreur : Nom déjà utilisé?".into(),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                });

                                let dialog_handle_cancel = dialog.as_weak();
                                dialog.on_cancelled(move || {
                                    if let Some(d) = dialog_handle_cancel.upgrade() {
                                        let _ = d.hide();
                                    }
                                });
                                let _ = dialog.run();
                            }
                        }
                        Err(e) => {
                            log::error!("Impossible de trouver l'utilisateur à éditer : {}", e)
                        }
                    }
                }
            }
        });

    // --- SUPPRESSION D'UN UTILISATEUR ---
    let delete_handle = main_window_handle.clone();
    main_window_handle
        .upgrade()
        .unwrap()
        .on_delete_user_clicked(move |user_id_str, username| {
            if let Some(ui) = delete_handle.upgrade() {
                if let Ok(dialog) = ui::DeleteUserDialog::new() {
                    dialog.set_username(username);

                    let main_ui_handle = ui.as_weak();
                    let dialog_handle = dialog.as_weak();

                    dialog.on_confirmed(move || {
                        if let Some(d) = dialog_handle.upgrade() {
                            if let Ok(user_id) = Uuid::parse_str(&user_id_str) {
                                if queries::delete_user(user_id).is_ok() {
                                    log::info!("Utilisateur {} supprimé.", user_id_str);
                                    if let Some(main_ui) = main_ui_handle.upgrade() {
                                        main_ui.invoke_request_users();
                                    }
                                }
                            }
                            let _ = d.hide();
                        }
                    });

                    let dialog_handle_cancel = dialog.as_weak();
                    dialog.on_cancelled(move || {
                        if let Some(d) = dialog_handle_cancel.upgrade() {
                            log::info!("Suppression annulée par l'utilisateur.");
                            let _ = d.hide(); // On ferme simplement la fenêtre
                        }
                    });
                    let _ = dialog.run();
                }
            }
        });

// --- RÉINITIALISATION DE MOT DE PASSE ---
main_window_handle.upgrade().unwrap().on_reset_password_clicked(move |user_id_str, username| { // On passe aussi le nom pour l'afficher
    if let Ok(dialog) = ui::ConfirmDialog::new() {
        dialog.set_dialog_title("Réinitialiser le Mot de Passe".into());
        dialog.set_message(
            format!(
                "Êtes-vous sûr de vouloir réinitialiser le mot de passe pour l'utilisateur '{}' ?\n\nUn nouveau mot de passe temporaire sera généré.",
                username
            ).into()
        );
        
        let dialog_handle = dialog.as_weak();
        dialog.on_confirmed(move || {
            // L'admin a confirmé, on procède à la réinitialisation
            if let Some(d) = dialog_handle.upgrade() {
                let _ = d.hide(); // Ferme la dialog de confirmation
                
                if let Ok(user_id) = Uuid::parse_str(&user_id_str) {
                    match queries::reset_user_password(user_id) {
                        Ok(temp_password) => {
                            // Afficher la popup d'information avec le mot de passe
                            if let Ok(info_dialog) = ui::InfoDialog::new() {
                                info_dialog.set_dialog_title("Mot de Passe Réinitialisé".into());
                                info_dialog.set_message(format!("Le mot de passe temporaire est :\n\n{}", temp_password).into());
                                let dialog_handle = info_dialog.as_weak();
                                info_dialog.on_ok_clicked(move || { 
                                    if let Some(d) = dialog_handle.upgrade() {
                                        let _ = d.hide(); 
                                    }
                                 });
                                let _ = info_dialog.run();
                            }
                        },
                        Err(e) => {
                            log::error!("Erreur lors de la réinitialisation du mot de passe: {}", e);
                            if let Ok(error_dialog) = ui::ErrorDialog::new() {
                                error_dialog.set_message("Impossible de réinitialiser le mot de passe.".into());
                                let dialog_handle_cancel = error_dialog.as_weak();
                                error_dialog.on_ok_clicked(move || { 
                                    if let Some(d) = dialog_handle_cancel.upgrade() {
                                        let _ = d.hide(); 
                                    }
                                });
                                let _ = error_dialog.run();
                            }
                        },
                    }
                }
            }
        });
        
        let dialog_handle_cancel = dialog.as_weak();
        dialog.on_cancelled(move || {
            // L'admin a annulé
            if let Some(d) = dialog_handle_cancel.upgrade() {
                log::info!("Réinitialisation annulée.");
                let _ = d.hide();
            }
        });

        let _ = dialog.run();
    }
});
}
