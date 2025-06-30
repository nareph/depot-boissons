// src/main_window_manager/product_callbacks.rs

use crate::{queries, ui};
use slint::{ComponentHandle, Weak};
use uuid::Uuid;

/// Configure tous les callbacks liés à la gestion des produits sur la fenêtre principale.
pub fn setup(main_window_handle: &Weak<ui::MainWindow>) {
   // --- CHARGEMENT DE LA LISTE DES PRODUITS ---
   let products_handle = main_window_handle.clone();
   main_window_handle
       .upgrade()
       .unwrap()
       .on_request_products(move || {
           if let Some(ui) = products_handle.upgrade() {
               log::info!("Chargement de la liste des produits...");
               match queries::get_all_products_with_offers() {
                   Ok(data) => {
                       let model = data
                           .into_iter()
                           .map(|(p, offers)| ui::ProductUI {
                               id: p.id.to_string().into(),
                               name: p.name.into(),
                               stock: format!(
                                   "{} {}",
                                   p.total_stock_in_base_units, p.base_unit_name
                               )
                               .into(),
                               price_offers: offers
                                   .into_iter()
                                   .map(|(o, u)| format!("{} XAF / {}", o.price, u.name))
                                   .collect::<Vec<_>>()
                                   .join("\n")
                                   .into(),
                           })
                           .collect::<Vec<_>>();
                       ui.set_products_model(
                           std::rc::Rc::new(slint::VecModel::from(model)).into(),
                       );
                   }
                   Err(e) => log::error!("Erreur lors du chargement des produits: {}", e),
               }
           }
       });

    // --- AJOUT D'UN PRODUIT ---
    let add_handle = main_window_handle.clone();
    main_window_handle.upgrade().unwrap().on_add_product_clicked(move || {
        if let Some(main_ui) = add_handle.upgrade() {
            if let Ok(dialog) = ui::AddProductDialog::new() {
                let main_ui_handle = main_ui.as_weak();
                let dialog_handle = dialog.as_weak();

                dialog.on_save_clicked(move |name, base_unit, stock| {
                    if let Some(d) = dialog_handle.upgrade() {
                        match queries::create_product(&name, &base_unit, stock) {
                            Ok(_) => {
                                if let Some(main) = main_ui_handle.upgrade() {
                                    main.invoke_request_products();
                                }
                                let _ = d.hide();
                            },
                            Err(e) => d.set_status_message(format!("Erreur: {}", e).into()),
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

    // --- ÉDITION D'UN PRODUIT ---
    let edit_handle = main_window_handle.clone();
    main_window_handle.upgrade().unwrap().on_edit_product_clicked(move |product_id_str| {
        if let Some(main_ui) = edit_handle.upgrade() {
            if let Ok(product_id) = Uuid::parse_str(&product_id_str) {
                if let Ok(product) = queries::get_product_by_id(product_id) {
                    if let Ok(dialog) = ui::EditProductDialog::new() {
                        dialog.set_product_id(product.id.to_string().into());
                        dialog.set_product_name(product.name.into());
                        dialog.set_base_unit_name(product.base_unit_name.into());
                        dialog.set_current_stock(product.total_stock_in_base_units);

                        let main_ui_handle = main_ui.as_weak();
                        let dialog_handle = dialog.as_weak();

                        dialog.on_save_clicked(move |id, name, base_unit, stock| {
                            if let Some(d) = dialog_handle.upgrade() {
                                let uuid = Uuid::parse_str(&id).unwrap();
                                match queries::update_product(uuid, &name, &base_unit, stock) {
                                    Ok(_) => {
                                        if let Some(main) = main_ui_handle.upgrade() {
                                            main.invoke_request_products();
                                        }
                                        let _ = d.hide();
                                    },
                                    Err(e) => d.set_status_message(format!("Erreur: {}", e).into()),
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
            }
        }
    });

    // --- SUPPRESSION D'UN PRODUIT ---
    let delete_handle = main_window_handle.clone();
    main_window_handle.upgrade().unwrap().on_delete_product_clicked(move |product_id_str, product_name| {
        if let Some(main_ui) = delete_handle.upgrade() {
            if let Ok(product_id) = Uuid::parse_str(&product_id_str) {
                match queries::can_delete_product(product_id) {
                    Ok(true) => {
                        if let Ok(dialog) = ui::DeleteProductDialog::new() {
                            dialog.set_product_name(product_name.clone());
                            let main_ui_handle = main_ui.as_weak();
                            let dialog_handle = dialog.as_weak();
                            
                            dialog.on_confirmed(move || {
                                if let Some(d) = dialog_handle.clone().upgrade() {
                                    if queries::delete_product(product_id).is_ok() {
                                        if let Some(main) = main_ui_handle.upgrade() {
                                            main.invoke_request_products();
                                        }
                                    }
                                    let _ = d.hide();
                                }
                            });
                            // On pourrait ajouter on_cancelled, mais ce n'est pas obligatoire
                            let _ = dialog.run();
                        }
                    },
                    Ok(false) => {
                        if let Ok(info_dialog) = ui::InfoDialog::new() {
                            info_dialog.set_dialog_title("Suppression impossible".into());
                            info_dialog.set_message(format!("Le produit '{}' ne peut pas être supprimé car il est lié à des ventes existantes.", product_name).into());
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
                        if let Ok(error_dialog) = ui::ErrorDialog::new() {
                            error_dialog.set_message(format!("Erreur: {}", e).into());
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