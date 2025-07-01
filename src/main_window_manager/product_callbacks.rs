// src/main_window_manager/product_callbacks.rs

use crate::{queries, ui};
use slint::{ComponentHandle, Weak};
use uuid::Uuid;
use std::sync::{Arc, Mutex};

/// Structure pour maintenir l'état des filtres et de la pagination
#[derive(Debug, Clone)]
pub struct ProductsState {
    pub search_query: String,
    pub stock_filter: queries::StockFilter,
    pub sort_by: queries::SortField,
    pub sort_order: queries::SortOrder,
    pub current_page: i64,
    pub page_size: i64,
}

impl Default for ProductsState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            stock_filter: queries::StockFilter::All,
            sort_by: queries::SortField::Name,
            sort_order: queries::SortOrder::Asc,
            current_page: 1,
            page_size: 10,
        }
    }
}

/// Configure tous les callbacks liés à la gestion des produits sur la fenêtre principale.
pub fn setup(main_window_handle: &Weak<ui::MainWindow>) {
    // État partagé pour les filtres et la pagination
    let products_state = Arc::new(Mutex::new(ProductsState::default()));

    // Fonction helper pour charger les produits avec les paramètres actuels
    let load_products = {
        let products_handle = main_window_handle.clone();
        let state = products_state.clone();
        
        move || {
            if let Some(ui) = products_handle.upgrade() {
                let current_state = state.lock().unwrap().clone();
                log::info!("Chargement des produits - Page: {}, Recherche: '{}'", 
                          current_state.current_page, current_state.search_query);
                
                let params = queries::ProductSearchParams::new()
                    .with_search(if current_state.search_query.is_empty() {
                        None
                    } else {
                        Some(current_state.search_query.clone())
                    })
                    .with_stock_filter(current_state.stock_filter.clone())
                    .with_sort(current_state.sort_by.clone(), current_state.sort_order.clone())
                    .with_pagination(current_state.current_page, current_state.page_size);
                
                match queries::get_products_paginated(params) {
                    Ok(paginated_result) => {
                        let model = paginated_result.products
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
                        ui.set_current_page(paginated_result.page as i32);
                        ui.set_total_pages(paginated_result.total_pages as i32);
                        ui.set_total_products(paginated_result.total_count as i32);
                        ui.set_products_per_page(paginated_result.page_size as i32);
                    }
                    Err(e) => {
                        log::error!("Erreur lors du chargement des produits: {}", e);
                        // Réinitialiser le modèle en cas d'erreur
                        ui.set_products_model(
                            std::rc::Rc::new(slint::VecModel::from(Vec::<ui::ProductUI>::new())).into(),
                        );
                        ui.set_total_products(0);
                        ui.set_current_page(1);
                        ui.set_total_pages(1);
                    }
                }
            }
        }
    };

    // --- CHARGEMENT INITIAL DE LA LISTE DES PRODUITS ---
    {
        let load_fn = load_products.clone();
        main_window_handle
            .upgrade()
            .unwrap()
            .on_request_products(move || {
                load_fn();
            });
    }

    // --- RECHERCHE DE PRODUITS ---
    {
        let load_fn = load_products.clone();
        let state = products_state.clone();
        let search_handle = main_window_handle.clone();
        
        main_window_handle
            .upgrade()
            .unwrap()
            .on_search_products(move |query| {
                if let Some(ui) = search_handle.upgrade() {
                    let mut current_state = state.lock().unwrap();
                    current_state.search_query = query.to_string();
                    current_state.current_page = 1; // Retour à la première page lors d'une recherche
                    ui.set_search_query(query);
                    drop(current_state);
                    load_fn();
                }
            });
    }

    // --- FILTRAGE PAR STOCK ---
    {
        let load_fn = load_products.clone();
        let state = products_state.clone();
        let filter_handle = main_window_handle.clone();
        
        main_window_handle
            .upgrade()
            .unwrap()
            .on_filter_products(move |filter_str| {
                if let Some(ui) = filter_handle.upgrade() {
                    let mut current_state = state.lock().unwrap();
                    current_state.stock_filter = match filter_str.as_str() {
                        "in_stock" => queries::StockFilter::InStock,
                        "out_of_stock" => queries::StockFilter::OutOfStock,
                        _ => queries::StockFilter::All,
                    };
                    current_state.current_page = 1; // Retour à la première page lors d'un filtrage
                    ui.set_stock_filter(filter_str);
                    drop(current_state);
                    load_fn();
                }
            });
    }

    // --- TRI DES PRODUITS ---
    {
        let load_fn = load_products.clone();
        let state = products_state.clone();
        let sort_handle = main_window_handle.clone();
        
        main_window_handle
            .upgrade()
            .unwrap()
            .on_sort_products(move |sort_field, sort_order| {
                if let Some(ui) = sort_handle.upgrade() {
                    let mut current_state = state.lock().unwrap();
                    current_state.sort_by = match sort_field.as_str() {
                        "stock" => queries::SortField::Stock,
                        "created_at" => queries::SortField::CreatedAt,
                        _ => queries::SortField::Name,
                    };
                    current_state.sort_order = match sort_order.as_str() {
                        "desc" => queries::SortOrder::Desc,
                        _ => queries::SortOrder::Asc,
                    };
                    ui.set_sort_by(sort_field);
                    ui.set_sort_order(sort_order);
                    drop(current_state);
                    load_fn();
                }
            });
    }

    // --- CHANGEMENT DE PAGE ---
    {
        let load_fn = load_products.clone();
        let state = products_state.clone();
        let page_handle = main_window_handle.clone();
        
        main_window_handle
            .upgrade()
            .unwrap()
            .on_change_page(move |page| {
                if let Some(_ui) = page_handle.upgrade() {
                    let mut current_state = state.lock().unwrap();
                    current_state.current_page = page as i64;
                    drop(current_state);
                    load_fn();
                }
            });
    }

    // --- CHANGEMENT DE TAILLE DE PAGE ---
    {
        let load_fn = load_products.clone();
        let state = products_state.clone();
        let page_size_handle = main_window_handle.clone();
        
        main_window_handle
            .upgrade()
            .unwrap()
            .on_change_page_size(move |size| {
                if let Some(ui) = page_size_handle.upgrade() {
                    let mut current_state = state.lock().unwrap();
                    current_state.page_size = size as i64;
                    current_state.current_page = 1; // Retour à la première page
                    ui.set_products_per_page(size);
                    drop(current_state);
                    load_fn();
                }
            });
    }

    // --- AJOUT D'UN PRODUIT ---
    {
        let load_fn = load_products.clone();
        let add_handle = main_window_handle.clone();
        
        main_window_handle.upgrade().unwrap().on_add_product_clicked(move || {
            if let Some(_main_ui) = add_handle.upgrade() {
                if let Ok(dialog) = ui::AddProductDialog::new() {
                    let dialog_handle = dialog.as_weak();
                    let load_fn_clone = load_fn.clone();

                    dialog.on_save_clicked(move |name, base_unit, stock| {
                        if let Some(d) = dialog_handle.upgrade() {
                            match queries::create_product(&name, &base_unit, stock) {
                                Ok(_) => {
                                    load_fn_clone();
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
    }

    // --- ÉDITION D'UN PRODUIT ---
    {
        let load_fn = load_products.clone();
        let edit_handle = main_window_handle.clone();
        
        main_window_handle.upgrade().unwrap().on_edit_product_clicked(move |product_id_str| {
            if let Some(_main_ui) = edit_handle.upgrade() {
                if let Ok(product_id) = Uuid::parse_str(&product_id_str) {
                    if let Ok(product) = queries::get_product_by_id(product_id) {
                        if let Ok(dialog) = ui::EditProductDialog::new() {
                            dialog.set_product_id(product.id.to_string().into());
                            dialog.set_product_name(product.name.into());
                            dialog.set_base_unit_name(product.base_unit_name.into());
                            dialog.set_current_stock(product.total_stock_in_base_units);

                            let dialog_handle = dialog.as_weak();
                            let load_fn_clone = load_fn.clone();

                            dialog.on_save_clicked(move |id, name, base_unit, stock| {
                                if let Some(d) = dialog_handle.upgrade() {
                                    let uuid = Uuid::parse_str(&id).unwrap();
                                    match queries::update_product(uuid, &name, &base_unit, stock) {
                                        Ok(_) => {
                                            load_fn_clone();
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
    }

    // --- SUPPRESSION D'UN PRODUIT ---
    {
        let load_fn = load_products.clone();
        let delete_handle = main_window_handle.clone();
        
        main_window_handle.upgrade().unwrap().on_delete_product_clicked(move |product_id_str, product_name| {
            if let Some(_main_ui) = delete_handle.upgrade() {
                if let Ok(product_id) = Uuid::parse_str(&product_id_str) {
                    match queries::can_delete_product(product_id) {
                        Ok(true) => {
                            if let Ok(dialog) = ui::DeleteProductDialog::new() {
                                dialog.set_product_name(product_name.clone());
                                let dialog_handle = dialog.as_weak();
                                let load_fn_clone = load_fn.clone();
                                
                                dialog.on_confirmed(move || {
                                    if let Some(d) = dialog_handle.clone().upgrade() {
                                        if queries::delete_product(product_id).is_ok() {
                                            load_fn_clone();
                                        }
                                        let _ = d.hide();
                                    }
                                });
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

    // Chargement initial des produits
    if let Some(ui) = main_window_handle.upgrade() {
        // Initialiser les propriétés de l'interface
        ui.set_current_page(1);
        ui.set_total_pages(1);
        ui.set_total_products(0);
        ui.set_products_per_page(10);
        ui.set_search_query("".into());
        ui.set_stock_filter("all".into());
        ui.set_sort_by("name".into());
        ui.set_sort_order("asc".into());
        
        // Charger les données
        load_products();
    }
}