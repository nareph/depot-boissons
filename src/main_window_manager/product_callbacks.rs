// src/main_window_manager/product_callbacks.rs

use crate::{queries, ui};
use bigdecimal::Num;
use slint::{ComponentHandle, Weak};
use uuid::Uuid;
use std::sync::{Arc, Mutex};

use super::{show_info_dialog, show_error_dialog};

/// Structure pour maintenir l'état des filtres et de la pagination
#[derive(Debug, Clone)]
pub struct ProductsState {
    pub search_query: String,
    pub stock_filter: queries::StockFilter,
    pub sort_by: queries::SortFieldProduct,
    pub sort_order: queries::SortOrder,
    pub current_page: i64,
    pub page_size: i64,
}

impl Default for ProductsState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            stock_filter: queries::StockFilter::All,
            sort_by: queries::SortFieldProduct::Name,
            sort_order: queries::SortOrder::Asc,
            current_page: 1,
            page_size: 5,
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
                log::info!("Chargement des produits - Page: {}, Recherche: '{}', Filtre: {:?}", 
                          current_state.current_page, current_state.search_query, current_state.stock_filter);
                
                let params = queries::ProductSearchParams::new()
                    .with_search(if current_state.search_query.is_empty() {
                        None
                    } else {
                        Some(current_state.search_query.clone())
                    })
                    .with_stock_filter(current_state.stock_filter)
                    .with_sort(current_state.sort_by, current_state.sort_order)
                    .with_pagination(current_state.current_page, current_state.page_size);
                
                match queries::get_products_paginated(params) {
                    Ok(paginated_result) => {
                        log::info!("Produits chargés: {} / {}", paginated_result.products.len(), paginated_result.total_count);
                        
                        let model = paginated_result.products
                            .into_iter()
                            .map(|p| ui::ProductUI {
                                id: p.id.to_string().into(),
                                name: p.name.into(),
                                stock: format!("{} ({})", p.stock_in_sale_units, p.packaging_description).into(),
                                price_offers: format!("{} XAF", p.price_per_sale_unit).into(),
                            })
                            .collect::<Vec<_>>();
                        
                        ui.set_products_model(
                            std::rc::Rc::new(slint::VecModel::from(model)).into(),
                        );
                        ui.set_product_current_page(paginated_result.page as i32);
                        ui.set_product_total_pages(paginated_result.total_pages as i32);
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
                        ui.set_product_current_page(1);
                        ui.set_product_total_pages(1);
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
                    
                    log::info!("Recherche de produits: '{}'", query);
                    
                    ui.set_product_search_query(query);
                    ui.set_product_current_page(1);
                    
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
                    
                    log::info!("Filtrage par stock: {}", filter_str);
                    
                    ui.set_product_stock_filter(filter_str);
                    ui.set_product_current_page(1);
                    
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
                        "stock" => queries::SortFieldProduct::Stock,
                        "price" => queries::SortFieldProduct::Price,
                        "created_at" => queries::SortFieldProduct::CreatedAt,
                        _ => queries::SortFieldProduct::Name,
                    };
                    current_state.sort_order = match sort_order.as_str() {
                        "desc" => queries::SortOrder::Desc,
                        _ => queries::SortOrder::Asc,
                    };
                    
                    log::info!("Tri par {} {}", sort_field, sort_order);
                    
                    ui.set_product_sort_by(sort_field);
                    ui.set_product_sort_order(sort_order);
                    
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
            .on_product_change_page(move |page| {
                if let Some(_ui) = page_handle.upgrade() {
                    let mut current_state = state.lock().unwrap();
                    current_state.current_page = page as i64;
                    
                    log::info!("Changement de page: {}", page);
                    
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
            .on_product_change_page_size(move |size| {
                if let Some(ui) = page_size_handle.upgrade() {
                    let mut current_state = state.lock().unwrap();
                    current_state.page_size = size as i64;
                    current_state.current_page = 1; // Retour à la première page
                    
                    log::info!("Changement de taille de page: {}", size);
                    
                    ui.set_products_per_page(size);
                    ui.set_product_current_page(1);
                    
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

                    dialog.on_save_clicked(move |name, packaging, stock, price_str| {
                        if let Some(d) = dialog_handle.upgrade() {
                            match bigdecimal::BigDecimal::from_str_radix(&price_str, 10) {
                                Ok(price) => {
                                    match queries::create_product(name.to_string(), packaging.to_string(), stock, price) {
                                        Ok(_) => {
                                            load_fn_clone();
                                            let _ = d.hide();
                                        },
                                        Err(e) => d.set_status_message(format!("Erreur: {}", e).into()),
                                    }
                                }
                                Err(_) => d.set_status_message("Le prix est invalide.".into()),
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
    }

    // --- ÉDITION D'UN PRODUIT ---
    {
        let load_fn = load_products.clone();
        let edit_handle = main_window_handle.clone();
        
        main_window_handle.upgrade().unwrap().on_edit_product_clicked(move |product_id_str| {
            if let Some(_main_ui) = edit_handle.upgrade() {
                if let Ok(product_id) = Uuid::parse_str(&product_id_str) {
                    if let Ok(product) = queries::get_product_by_id(&product_id.to_string()) {
                        if let Ok(dialog) = ui::EditProductDialog::new() {
                            dialog.set_product_id(product.id.to_string().into());
                            dialog.set_product_name(product.name.into());
                            dialog.set_packaging_description(product.packaging_description.into());
                            dialog.set_current_stock(product.stock_in_sale_units);
                            dialog.set_price(product.price_per_sale_unit.to_string().into());

                            let dialog_handle = dialog.as_weak();
                            let load_fn_clone = load_fn.clone();

                            dialog.on_save_clicked(move |id, name, packaging, stock, price_str| {
                                if let Some(d) = dialog_handle.upgrade() {
                                    if let (Ok(uuid), Ok(price)) = (Uuid::parse_str(&id), bigdecimal::BigDecimal::from_str_radix(&price_str, 10)) {
                                        match queries::update_product(&uuid.to_string(), name.to_string(), packaging.to_string(), stock, price) {
                                            Ok(_) => {
                                                load_fn_clone();
                                                let _ = d.hide();
                                            },
                                            Err(e) => d.set_status_message(format!("Erreur: {}", e).into()),
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
                    let id = product_id.to_string();
                    match queries::can_delete_product(&id) {
                        Ok(true) => {
                            if let Ok(dialog) = ui::DeleteProductDialog::new() {
                                dialog.set_product_name(product_name.clone());
                                let dialog_handle = dialog.as_weak();
                                let load_fn_clone = load_fn.clone();
                                
                                dialog.on_ok_clicked(move || {
                                    if let Some(d) = dialog_handle.clone().upgrade() {
                                        if queries::delete_product(&id).is_ok() {
                                            load_fn_clone();
                                        }
                                        let _ = d.hide();
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
                        },
                        Ok(false) => {
                            show_info_dialog(
                                "Suppression impossible",
                                &format!("Le produit '{}' ne peut pas être supprimé car il est lié à des ventes existantes.", product_name)
                            );
                        },
                        Err(e) => {
                            show_error_dialog(
                                "Erreur de suppression",
                                &format!("Impossible de vérifier la suppression du produit '{}': {}", product_name, e)
                            );
                        }
                    }
                }
            }
        });
    }

    // Chargement initial des produits
    if let Some(ui) = main_window_handle.upgrade() {
        // Initialiser les propriétés de l'interface
        ui.set_product_current_page(1);
        ui.set_product_total_pages(1);
        ui.set_total_products(0);
        ui.set_products_per_page(5);
        ui.set_product_search_query("".into());
        ui.set_product_stock_filter("all".into());
        ui.set_product_sort_by("name".into());
        ui.set_product_sort_order("asc".into());
        
        log::info!("Chargement initial des produits");
        // Charger les données
        load_products();
    }
}