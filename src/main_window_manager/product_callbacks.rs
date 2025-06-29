// src/main_window_manager/product_callbacks.rs

use crate::{queries, ui};
use slint::{ComponentHandle, Model, SharedString, Weak};
use uuid::Uuid;

/// Configure tous les callbacks liés à la gestion des produits sur la fenêtre principale.
pub fn setup(main_window_handle: &Weak<ui::MainWindow>) {
    let main_window = main_window_handle.upgrade().unwrap();

    let products_handle = main_window_handle.clone();
    main_window.on_request_products(move || {
        if let Some(ui) = products_handle.upgrade() {
            log::info!("Chargement des produits..."); 
            
            match queries::get_all_products_with_offers() {
                Ok(data) => {
                    let model = data
                        .into_iter()
                        .map(|(p, offers)| ui::ProductUI {
                            id: p.id.to_string().into(),
                            name: p.name.into(),
                            stock: format!("{} {}", p.total_stock_in_base_units, p.base_unit_name).into(),
                            price_offers: offers.into_iter().map(|(o, u)| format!("{} XAF / {}", o.price, u.name)).collect::<Vec<_>>().join("\n").into(),
                        })
                        .collect::<Vec<_>>();
                    ui.set_products_model(std::rc::Rc::new(slint::VecModel::from(model)).into());
                },
                Err(e) => log::error!("Erreur chargement produits: {}", e),
            }
        }
    });

    // --- Callback pour l'ajout d'un produit ---
    let add_handle = main_window_handle.clone();
    main_window.on_add_product_clicked(move || {
        if let Some(main_ui) = add_handle.upgrade() {
            if let Ok(dialog) = ui::AddProductDialog::new() {
                let main_ui_handle = main_ui.as_weak();
                let dialog_handle = dialog.as_weak();

                dialog.on_save_clicked(
                    move |name: SharedString, base_unit: SharedString, stock: i32| {
                        if let Some(d) = dialog_handle.upgrade() {
                            match queries::create_product(&name, &base_unit, stock) {
                                Ok(_) => {
                                    log::info!("Produit '{}' créé avec succès.", name);
                                    
                                    // CORRECTION: Fermer explicitement le dialogue
                                    if let Err(e) = d.hide() {
                                        log::warn!("Erreur lors de la fermeture du dialogue: {}", e);
                                    }
                                    
                                    // Rafraîchir la liste
                                    if let Some(main) = main_ui_handle.upgrade() {
                                        main.invoke_request_products();
                                    }
                                }
                                Err(e) => {
                                    log::error!("Erreur lors de la création du produit: {}", e);
                                    d.set_status_message(format!("Erreur: {}", e).into());
                                    // NE PAS fermer le dialogue en cas d'erreur pour permettre à l'utilisateur de corriger
                                }
                            }
                        }
                    },
                );

                let _ = dialog.run();
            }
        }
    });

    // --- CORRECTION: Callback pour l'édition d'un produit ---
    let edit_handle = main_window_handle.clone();
    main_window.on_edit_product_clicked(move |index| {
        if let Some(main_ui) = edit_handle.upgrade() {
            let products_model = main_ui.get_products_model();

            if let Some(product_ui) = products_model.row_data(index as usize) {
                if let Ok(product_id) = Uuid::parse_str(&product_ui.id) {
                    if let Ok(product) = queries::get_product_by_id(product_id) {
                        if let Ok(dialog) = ui::EditProductDialog::new() {
                            dialog.set_product_id(product.id.to_string().into());
                            dialog.set_product_name(product.name.into());
                            dialog.set_base_unit_name(product.base_unit_name.into());
                            dialog.set_current_stock(product.total_stock_in_base_units);

                            let main_ui_handle = main_ui.as_weak();
                            let dialog_handle = dialog.as_weak();

                            dialog.on_save_clicked(
                                move |id: SharedString,
                                      name: SharedString,
                                      base_unit: SharedString,
                                      stock: i32| {
                                    if let Some(d) = dialog_handle.upgrade() {
                                        let product_id = Uuid::parse_str(&id).unwrap();
                                        match queries::update_product(
                                            product_id, &name, &base_unit, stock,
                                        ) {
                                            Ok(_) => {
                                                log::info!(
                                                    "Produit '{}' mis à jour avec succès.",
                                                    name
                                                );
                                                
                                                // Fermer explicitement le dialogue
                                                if let Err(e) = d.hide() {
                                                    log::warn!("Erreur lors de la fermeture du dialogue: {}", e);
                                                }
                                                
                                                // Rafraîchir la liste
                                                if let Some(main) = main_ui_handle.upgrade() {
                                                    main.invoke_request_products();
                                                }
                                            }
                                            Err(e) => {
                                                log::error!(
                                                    "Erreur lors de la mise à jour du produit: {}",
                                                    e
                                                );
                                                d.set_status_message(format!("Erreur: {}", e).into());
                                                // NE PAS fermer le dialogue en cas d'erreur
                                            }
                                        }
                                    }
                                },
                            );

                            let _ = dialog.run();
                        }
                    }
                }
            }
        }
    });

    // --- Callback pour la suppression d'un produit ---
    let delete_handle = main_window_handle.clone();
    main_window_handle.upgrade().unwrap().on_delete_product_clicked(move |index| {
        if let Some(main_ui) = delete_handle.upgrade() {
            let products_model = main_ui.get_products_model();
            if let Some(product_ui) = products_model.row_data(index as usize) {
                let product_id = Uuid::parse_str(&product_ui.id).unwrap();
                let product_name = product_ui.name.to_string();

                match queries::can_delete_product(product_id) {
                    Ok(true) => {
                        // Le produit peut être supprimé
                        if let Ok(dialog) = ui::DeleteProductDialog::new() {
                            dialog.set_product_name(product_name.clone().into());
                            
                            let main_ui_handle = main_ui.as_weak();
                            let dialog_handle = dialog.as_weak();

                            // Cloner les références pour les closures
                            let confirmed_dialog_handle = dialog_handle.clone();
                            dialog.on_confirmed(move || {
                                if let Some(d) = confirmed_dialog_handle.upgrade() {
                                    match queries::delete_product(product_id) {
                                        Ok(count) if count > 0 => {
                                            log::info!("Produit '{}' supprimé.", product_name);
                                            if let Some(main) = main_ui_handle.upgrade() {
                                                main.invoke_request_products();
                                            }
                                        },
                                        _ => log::error!("La suppression a échoué."),
                                    }
                                    let _ = d.hide(); // Fermer le dialogue
                                }
                            });

                            let cancelled_dialog_handle = dialog_handle.clone();
                            dialog.on_cancelled(move || {
                                if let Some(d) = cancelled_dialog_handle.upgrade() {
                                    log::info!("Suppression annulée.");
                                    let _ = d.hide(); // Fermer le dialogue
                                }
                            });

                            let _ = dialog.run();
                        }
                    },
                    Ok(false) => {
                        log::warn!("Tentative de suppression du produit '{}', mais il a des ventes associées.", product_name);
                        
                        // Créer un dialogue d'information pour l'utilisateur
                        if let Ok(info_dialog) = ui::InfoDialog::new() {
                            info_dialog.set_dialog_title("Suppression impossible".into());
                            info_dialog.set_message(
                                format!(
                                    "Le produit '{}' ne peut pas être supprimé car il est associé à des ventes existantes.\n\nPour supprimer ce produit, vous devez d'abord supprimer toutes les ventes qui lui sont associées.",
                                    product_name
                                ).into()
                            );
                            
                            let info_dialog_handle = info_dialog.as_weak();
                            info_dialog.on_ok_clicked(move || {
                                if let Some(d) = info_dialog_handle.upgrade() {
                                    let _ = d.hide();
                                }
                            });
                            
                            let _ = info_dialog.run();
                        }
                    },
                    Err(e) => {
                        log::error!("Erreur lors de la vérification de suppression: {}", e);
                        
                        if let Ok(error_dialog) = ui::ErrorDialog::new() {
                            error_dialog.set_dialog_title("Erreur".into());
                            error_dialog.set_message(
                                format!("Une erreur s'est produite lors de la vérification: {}", e).into()
                            );
                            
                            let error_dialog_handle = error_dialog.as_weak();
                            error_dialog.on_ok_clicked(move || {
                                if let Some(d) = error_dialog_handle.upgrade() {
                                    let _ = d.hide();
                                }
                            });
                            
                            let _ = error_dialog.run();
                        }
                    }
                }
            }
        }
    });
}