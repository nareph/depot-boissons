// src/main_window_manager/user_callbacks.rs

use crate::{queries, ui};
use slint::{ComponentHandle, Model, Weak};
use uuid::Uuid;
use std::cell::RefCell;
use std::rc::Rc;

/// Configure tous les callbacks liés à la gestion des utilisateurs pour la fenêtre principale.
/// Cette fonction n'est appelée que si l'utilisateur connecté est un administrateur.
pub fn setup(main_window_handle: &Weak<ui::MainWindow>, current_user_id: Uuid) {
    // État partagé de la pagination et du filtrage
    let current_state = Rc::new(RefCell::new((
        queries::user_queries::UserFilter::default(),
        queries::user_queries::UserPagination::default(),
    )));

    // --- FONCTION UTILITAIRE POUR CHARGER LES UTILISATEURS AVEC FILTRES ---
    let load_users_with_filters = {
        let state = current_state.clone();
        move |ui: &ui::MainWindow, 
              filter: &queries::user_queries::UserFilter, 
              pagination: &queries::user_queries::UserPagination| {
            
            log::info!("load_users_with_filters appelé avec: search_term={:?}, role_filter={:?}, page={}, per_page={}", 
                      filter.search_term, filter.role_filter, pagination.page, pagination.per_page);
            
            // Mettre à jour l'état partagé
            {
                let mut state_borrow = state.borrow_mut();
                state_borrow.0 = filter.clone();
                state_borrow.1 = pagination.clone();
            }

            match queries::user_queries::get_users_paginated(current_user_id, filter.clone(), pagination.clone()) {
                Ok(result) => {
                    let users_count = result.users.len();
                    log::info!("Résultat de la requête: {} utilisateurs trouvés, page {}/{}, total: {}", 
                              users_count, result.current_page, result.total_pages, result.total_count);
                    
                    let model = result.users
                        .into_iter()
                        .map(|u| {
                            log::debug!("Utilisateur trouvé: {} ({})", u.name, u.role);
                            ui::UserUI {
                                id: u.id.to_string().into(),
                                name: u.name.into(),
                                role: u.role.into(),
                            }
                        })
                        .collect::<Vec<_>>();
                    
                    ui.set_users_model(std::rc::Rc::new(slint::VecModel::from(model)).into());
                    
                    // Mettre à jour les propriétés de la vue
                    ui.set_user_current_page(result.current_page as i32);
                    ui.set_user_total_pages(result.total_pages as i32);
                    ui.set_user_total_users(result.total_count as i32);
                    ui.set_users_per_page(result.per_page as i32);
                    
                    log::info!("UI mise à jour avec {} utilisateurs (page {}/{})", 
                              users_count, result.current_page, result.total_pages);
                }
                Err(e) => {
                    log::error!("Erreur lors du chargement des utilisateurs: {}", e);
                    ui.set_users_model(std::rc::Rc::new(slint::VecModel::default()).into());
                }
            }
        }
    };

    // --- CHARGEMENT INITIAL DE LA LISTE DES UTILISATEURS ---
    let users_handle = main_window_handle.clone();
    let load_users_fn = load_users_with_filters.clone();
    let state_clone = current_state.clone();
    main_window_handle
        .upgrade()
        .unwrap()
        .on_request_users(move || {
            if let Some(ui) = users_handle.upgrade() {
                log::info!("Chargement de la liste des utilisateurs...");
                
                // Lire l'état actuel dans un scope limité
                let (filter, pagination) = {
                    let state_borrow = state_clone.borrow();
                    (state_borrow.0.clone(), state_borrow.1.clone())
                };
                
                load_users_fn(&ui, &filter, &pagination);
            }
        });

    // --- RECHERCHE D'UTILISATEURS ---
    let search_handle = main_window_handle.clone();
    let load_users_fn = load_users_with_filters.clone();
    let state_clone = current_state.clone();
    main_window_handle
        .upgrade()
        .unwrap()
        .on_search_users(move |search_term| {
            if let Some(ui) = search_handle.upgrade() {
                log::info!("Recherche d'utilisateurs avec le terme: '{}'", search_term);
                
                // Mettre à jour l'état dans un scope limité
                let (filter, pagination) = {
                    let mut state_borrow = state_clone.borrow_mut();
                    let (ref mut filter, ref mut pagination) = *state_borrow;
                    
                    // Mettre à jour le terme de recherche
                    filter.search_term = if search_term.trim().is_empty() { 
                        None 
                    } else { 
                        Some(search_term.to_string()) 
                    };
                    
                    // Retour à la première page lors d'une recherche
                    pagination.page = 1;
                    
                    log::info!("État après recherche: search_term={:?}, role_filter={:?}", 
                              filter.search_term, filter.role_filter);
                    
                    (filter.clone(), pagination.clone())
                };
                
                // Mettre à jour l'UI
                ui.set_user_search_query(search_term);
                
                load_users_fn(&ui, &filter, &pagination);
            }
        });

    // --- FILTRAGE PAR RÔLE ---
    let filter_handle = main_window_handle.clone();
    let load_users_fn = load_users_with_filters.clone();
    let state_clone = current_state.clone();
    main_window_handle
        .upgrade()
        .unwrap()
        .on_filter_users(move |role_filter| {
            if let Some(ui) = filter_handle.upgrade() {
                log::info!("Filtrage par rôle: '{}'", role_filter);
                
                // Mettre à jour l'état dans un scope limité
                let (filter, pagination) = {
                    let mut state_borrow = state_clone.borrow_mut();
                    let (ref mut filter, ref mut pagination) = *state_borrow;
                    
                    // Mettre à jour le filtre de rôle
                    filter.role_filter = if role_filter == "all" { 
                        None 
                    } else { 
                        Some(role_filter.to_string()) 
                    };
                    
                    // Retour à la première page lors d'un filtrage
                    pagination.page = 1;
                    
                    log::info!("État après filtrage: search_term={:?}, role_filter={:?}", 
                              filter.search_term, filter.role_filter);
                    
                    (filter.clone(), pagination.clone())
                };
                
                // Mettre à jour l'UI
                ui.set_role_filter(role_filter);
                
                load_users_fn(&ui, &filter, &pagination);
            }
        });

    // --- TRI DES UTILISATEURS ---
    let sort_handle = main_window_handle.clone();
    let load_users_fn = load_users_with_filters.clone();
    let state_clone = current_state.clone();
    main_window_handle
        .upgrade()
        .unwrap()
        .on_sort_users(move |sort_by, sort_order| {
            if let Some(ui) = sort_handle.upgrade() {
                log::info!("Tri par: '{}' ordre: '{}'", sort_by, sort_order);
                
                // Mettre à jour l'état dans un scope limité
                let (filter, pagination) = {
                    let mut state_borrow = state_clone.borrow_mut();
                    let (ref mut filter, ref pagination) = *state_borrow;
                    
                    // Mettre à jour les critères de tri
                    filter.sort_by = match sort_by.as_str() {
                        "role" => queries::user_queries::UserSortBy::Role,
                        "created_at" => queries::user_queries::UserSortBy::CreatedAt,
                        _ => queries::user_queries::UserSortBy::Name,
                    };
                    filter.sort_order = match sort_order.as_str() {
                        "desc" => queries::SortOrder::Desc,
                        _ => queries::SortOrder::Asc,
                    };
                    
                    (filter.clone(), pagination.clone())
                };
                
                // Mettre à jour l'UI
                ui.set_user_sort_by(sort_by);
                ui.set_user_sort_order(sort_order);
                
                load_users_fn(&ui, &filter, &pagination);
            }
        });

    // --- CHANGEMENT DE PAGE ---
    let page_handle = main_window_handle.clone();
    let load_users_fn = load_users_with_filters.clone();
    let state_clone = current_state.clone();
    main_window_handle
        .upgrade()
        .unwrap()
        .on_user_change_page(move |new_page| {
            if let Some(ui) = page_handle.upgrade() {
                log::info!("Changement vers la page: {}", new_page);
                
                // Mettre à jour l'état dans un scope limité
                let (filter, pagination) = {
                    let mut state_borrow = state_clone.borrow_mut();
                    let (ref filter, ref mut pagination) = *state_borrow;
                    
                    // Mettre à jour la page
                    pagination.page = new_page as i64;
                    
                    (filter.clone(), pagination.clone())
                };
                
                load_users_fn(&ui, &filter, &pagination);
            }
        });

    // --- CHANGEMENT DU NOMBRE D'ÉLÉMENTS PAR PAGE ---
    let page_size_handle = main_window_handle.clone();
    let load_users_fn = load_users_with_filters.clone();
    let state_clone = current_state.clone();
    main_window_handle
        .upgrade()
        .unwrap()
        .on_user_change_page_size(move |new_page_size| {
            if let Some(ui) = page_size_handle.upgrade() {
                log::info!("Changement du nombre d'éléments par page: {}", new_page_size);
                
                // Mettre à jour l'état dans un scope limité
                let (filter, pagination) = {
                    let mut state_borrow = state_clone.borrow_mut();
                    let (ref filter, ref mut pagination) = *state_borrow;
                    
                    // Mettre à jour la taille de page et revenir à la première page
                    pagination.per_page = new_page_size as i64;
                    pagination.page = 1;
                    
                    (filter.clone(), pagination.clone())
                };
                
                load_users_fn(&ui, &filter, &pagination);
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
                                if queries::user_queries::create_user(&name, &hash, &role).is_ok() {
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
                    dialog.on_cancel_clicked(move || {
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
                    match queries::user_queries::get_user_by_id(user_id) {
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
                                            match queries::user_queries::update_user_info(
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
                                dialog.on_cancel_clicked(move || {
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

                    dialog.on_ok_clicked(move || {
                        if let Some(d) = dialog_handle.upgrade() {
                            if let Ok(user_id) = Uuid::parse_str(&user_id_str) {
                                if queries::user_queries::delete_user(user_id).is_ok() {
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
                    dialog.on_cancel_clicked(move || {
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
    main_window_handle
        .upgrade()
        .unwrap()
        .on_reset_password_clicked(move |user_id_str, username| {
            if let Ok(dialog) = ui::ConfirmDialog::new() {
                dialog.set_dialog_title("Réinitialiser le Mot de Passe".into());
                dialog.set_message(
                    format!(
                        "Êtes-vous sûr de vouloir réinitialiser le mot de passe pour l'utilisateur '{}' ?\n\nUn nouveau mot de passe temporaire sera généré.",
                        username
                    ).into()
                );
                
                let dialog_handle = dialog.as_weak();
                dialog.on_ok_clicked(move || {
                    // L'admin a confirmé, on procède à la réinitialisation
                    if let Some(d) = dialog_handle.upgrade() {
                        let _ = d.hide(); // Ferme la dialog de confirmation
                        
                        if let Ok(user_id) = Uuid::parse_str(&user_id_str) {
                            match queries::user_queries::reset_user_password(user_id) {
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
                dialog.on_cancel_clicked(move || {
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