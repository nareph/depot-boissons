use crate::{
    models::{CreateSaleData, CreateSaleItemData, Receipt},
    queries::{self},
    ui,
};
use bigdecimal::BigDecimal;
use slint::{ComponentHandle, ModelRc, Weak};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

use super::{show_error_dialog, show_info_dialog};

/// Maintient l'état des filtres et pagination pour les ventes
#[derive(Debug, Clone)]
pub struct SalesState {
    pub search_query: String,
    pub date_filter: queries::DateFilter,
    pub sort_by: queries::SortFieldSale,
    pub sort_order: queries::SortOrder,
    pub current_page: i64,
    pub page_size: i64,
    pub current_user_id: Uuid,
    pub is_admin: bool,
}

impl SalesState {
    pub fn new(current_user_id: Uuid, is_admin: bool) -> Self {
        Self {
            search_query: String::new(),
            date_filter: queries::DateFilter::All,
            sort_by: queries::SortFieldSale::Date,
            sort_order: queries::SortOrder::Desc,
            current_page: 1,
            page_size: 5,
            current_user_id,
            is_admin,
        }
    }
}

// Structure pour gérer l'état d'une nouvelle vente
#[derive(Debug, Clone)]
pub struct NewSaleState {
    pub cart_items: HashMap<String, CartItemData>,
    pub total_amount: BigDecimal,
}

#[derive(Debug, Clone)]
pub struct CartItemData {
    pub product_id: String,
    pub product_name: String,
    pub unit_price: BigDecimal,
    pub quantity: i32,
    pub total_price: BigDecimal,
    pub packaging_description: String,
}

impl NewSaleState {
    pub fn new() -> Self {
        Self {
            cart_items: HashMap::new(),
            total_amount: BigDecimal::from(0),
        }
    }

    pub fn add_item(
        &mut self,
        product_id: String,
        product_name: String,
        unit_price: BigDecimal,
        quantity: i32,
        packaging_description: String,
    ) {
        let total_price = unit_price.clone() * BigDecimal::from(quantity);

        if let Some(existing_item) = self.cart_items.get_mut(&product_id) {
            // Si l'article existe déjà, augmenter la quantité
            existing_item.quantity += quantity;
            existing_item.total_price =
                existing_item.unit_price.clone() * BigDecimal::from(existing_item.quantity);
        } else {
            // Nouvel article
            self.cart_items.insert(
                product_id.clone(),
                CartItemData {
                    product_id,
                    product_name,
                    unit_price,
                    quantity,
                    total_price,
                    packaging_description,
                },
            );
        }

        self.update_total();
    }

    pub fn remove_item(&mut self, product_id: &str) {
        self.cart_items.remove(product_id);
        self.update_total();
    }

    pub fn update_quantity(&mut self, product_id: &str, new_quantity: i32) {
        if let Some(item) = self.cart_items.get_mut(product_id) {
            item.quantity = new_quantity;
            item.total_price = item.unit_price.clone() * BigDecimal::from(new_quantity);
            self.update_total();
        }
    }

    pub fn clear_cart(&mut self) {
        self.cart_items.clear();
        self.update_total();
    }

    fn update_total(&mut self) {
        self.total_amount = self
            .cart_items
            .values()
            .map(|item| item.total_price.clone())
            .sum();
    }

    pub fn to_cart_ui_items(&self) -> Vec<ui::CartItem> {
        self.cart_items
            .values()
            .map(|item| ui::CartItem {
                product_id: item.product_id.clone().into(),
                product_name: item.product_name.clone().into(),
                unit_price: format!("{:.0}", item.unit_price).into(),
                quantity: item.quantity,
                total_price: format!("{:.0}", item.total_price).into(),
                packaging_description: item.packaging_description.clone().into(),
            })
            .collect()
    }
}

/// Configure les callbacks pour la gestion des ventes
pub fn setup(main_window_handle: &Weak<ui::MainWindow>, current_user_id: Uuid, is_admin: bool) {
    let sales_state = Arc::new(Mutex::new(SalesState::new(current_user_id, is_admin)));

    // Fonction pour charger les ventes
    let load_sales = {
        let handle = main_window_handle.clone();
        let state_clone = sales_state.clone();

        move || {
            if let Some(ui) = handle.upgrade() {
                let current_state = state_clone.lock().unwrap().clone();

                let params = queries::SaleSearchParams {
                    user_id_filter: if current_state.is_admin {
                        None
                    } else {
                        Some(current_state.current_user_id)
                    },
                    search_query: if current_state.search_query.is_empty() {
                        None
                    } else {
                        Some(current_state.search_query.clone())
                    },
                    date_filter: current_state.date_filter,
                    sort_by: current_state.sort_by,
                    sort_order: current_state.sort_order,
                    page: current_state.current_page,
                    page_size: current_state.page_size,
                };

                match queries::get_sales_paginated(params) {
                    Ok(result) => {
                        let model = result
                            .sales
                            .into_iter()
                            .map(|s| ui::SaleUI {
                                id: s.sale.id.to_string().into(),
                                sale_number: s.sale.sale_number.into(),
                                date: s.sale.date.format("%d/%m/%Y %H:%M").to_string().into(),
                                total_amount: format!("{} XAF", s.sale.total_amount).into(),
                                seller_name: s.seller_name.into(),
                                items_count: s.items_count as i32,
                                items: ModelRc::new(slint::VecModel::default()),
                                show_details: false,
                            })
                            .collect::<Vec<_>>();

                        ui.set_sales_model(ModelRc::new(slint::VecModel::from(model)));
                        ui.set_sale_current_page(result.page as i32);
                        ui.set_sale_total_pages(result.total_pages as i32);
                        ui.set_total_sales(result.total_count as i32);
                        ui.set_is_admin(current_state.is_admin);
                    }
                    Err(e) => {
                        show_error_dialog(
                            "Erreur de chargement",
                            &format!("Impossible de charger les ventes: {}", e),
                        );
                        ui.set_sales_model(ModelRc::new(slint::VecModel::default()));
                    }
                }
            }
        }
    };

    let ui = main_window_handle.upgrade().unwrap();

    // Callbacks de base
    ui.on_request_sales({
        let load_fn = load_sales.clone();
        move || load_fn()
    });

    ui.on_search_sales({
        let state = sales_state.clone();
        let load_fn = load_sales.clone();
        move |query| {
            let mut s = state.lock().unwrap();
            s.search_query = query.to_string();
            s.current_page = 1; // Reset à la page 1 lors d'une recherche
            drop(s);
            load_fn();
        }
    });

    ui.on_filter_sales({
        let state = sales_state.clone();
        let load_fn = load_sales.clone();
        move |filter| {
            let mut s = state.lock().unwrap();
            s.date_filter = match filter.as_str() {
                "today" => queries::DateFilter::Today,
                "week" => queries::DateFilter::Week,
                "month" => queries::DateFilter::Month,
                _ => queries::DateFilter::All,
            };
            s.current_page = 1; // Reset à la page 1 lors du filtrage
            drop(s);
            load_fn();
        }
    });

    // Callback pour le tri
    ui.on_sort_sales({
        let state = sales_state.clone();
        let load_fn = load_sales.clone();
        move |sort_by, sort_order| {
            let mut s = state.lock().unwrap();
            s.sort_by = match sort_by.as_str() {
                "date" => queries::SortFieldSale::Date,
                "amount" => queries::SortFieldSale::Amount,
                "sale_number" => queries::SortFieldSale::SaleNumber,
                _ => queries::SortFieldSale::Date,
            };
            s.sort_order = match sort_order.as_str() {
                "asc" => queries::SortOrder::Asc,
                "desc" => queries::SortOrder::Desc,
                _ => queries::SortOrder::Desc,
            };
            s.current_page = 1; // Reset à la page 1 lors du tri
            drop(s);
            load_fn();
        }
    });

    // Callback pour la pagination
    ui.on_sale_change_page({
        let state = sales_state.clone();
        let load_fn = load_sales.clone();
        move |page| {
            let mut s = state.lock().unwrap();
            s.current_page = page as i64;
            drop(s);
            load_fn();
        }
    });

    // Callback pour changer la taille de page
    ui.on_sale_change_page_size({
        let state = sales_state.clone();
        let load_fn = load_sales.clone();
        move |page_size| {
            let mut s = state.lock().unwrap();
            s.page_size = page_size as i64;
            s.current_page = 1; // Reset à la page 1 lors du changement de taille
            drop(s);
            load_fn();
        }
    });

    // Callback pour ajouter une vente
    ui.on_add_sale_clicked({
        let ui_handle = main_window_handle.clone();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                show_new_sale_dialog(&ui, current_user_id);
            }
        }
    });

    // Gestion des détails de vente
    ui.on_view_sale_details({
        let state = sales_state.clone();
        let ui_handle = main_window_handle.clone();
        move |sale_id_str| {
            if let Some(ui) = ui_handle.upgrade() {
                let current_state = state.lock().unwrap().clone();

                match Uuid::parse_str(&sale_id_str) {
                    Ok(sale_id) => {
                        match queries::get_sale_details(
                            sale_id,
                            current_state.current_user_id,
                            current_state.is_admin,
                        ) {
                            Ok(sale_details) => {
                                let items_ui = sale_details
                                    .items
                                    .into_iter()
                                    .map(|(item, product)| ui::SaleItemUI {
                                        id: item.id.to_string().into(),
                                        product_name: product.name.into(),
                                        packaging_description: product.packaging_description.into(),
                                        quantity: item.quantity,
                                        unit_price: format!("{} XAF", item.unit_price).into(),
                                        total_price: format!("{} XAF", item.total_price).into(),
                                    })
                                    .collect::<Vec<_>>();

                                let details_dialog = ui::SaleDetailsDialog::new().unwrap();
                                details_dialog.set_sale_details(ui::SaleDetailsUI {
                                    id: sale_details.sale.id.to_string().into(),
                                    sale_number: sale_details.sale.sale_number.into(),
                                    date: sale_details
                                        .sale
                                        .date
                                        .format("%d/%m/%Y %H:%M")
                                        .to_string()
                                        .into(),
                                    total_amount: format!("{} XAF", sale_details.sale.total_amount)
                                        .into(),
                                    seller_name: sale_details.seller_name.into(),
                                    items: ModelRc::new(slint::VecModel::from(items_ui)),
                                });

                                // Gestion de l'impression
                                let ui_weak = ui.as_weak();
                                details_dialog.on_print_clicked(move || {
                                    if let Some(_ui) = ui_weak.upgrade() {
                                        match queries::generate_receipt(sale_id) {
                                            Ok(receipt) => {
                                                show_receipt_dialog(receipt);
                                            }
                                            Err(e) => {
                                                show_error_dialog(
                                                    "Erreur",
                                                    &format!(
                                                        "Impossible de générer le reçu: {}",
                                                        e
                                                    ),
                                                );
                                            }
                                        }
                                    }
                                });

                                let details_dialog_weak = details_dialog.as_weak();
                                details_dialog.on_close_clicked(move || {
                                    if let Some(dd) = details_dialog_weak.upgrade() {
                                        let _ = dd.hide();
                                    }
                                });

                                let _ = details_dialog.run();
                            }
                            Err(e) => show_error_dialog("Erreur", &format!("{}", e)),
                        }
                    }
                    Err(e) => show_error_dialog("Erreur", &format!("ID invalide: {}", e)),
                }
            }
        }
    });
    // Chargement initial
    load_sales();
}

// Fonction utilitaire pour afficher le ticket de caisse
fn show_receipt_dialog(receipt: Receipt) {
    let receipt_dialog = ui::ReceiptDialog::new().unwrap();
    receipt_dialog.set_sale_number(receipt.sale_number.into());
    receipt_dialog.set_date(receipt.date.into());
    receipt_dialog.set_seller_name(receipt.seller_name.into());
    receipt_dialog.set_total_amount(receipt.total_amount.to_string().into());

    let items = receipt
        .items
        .into_iter()
        .map(|item| ui::ReceiptItemUI {
            product_name: item.product_name.into(),
            quantity: item.quantity.to_string().into(),
            unit_price: item.unit_price.to_string().into(),
            total_price: item.total_price.to_string().into(),
        })
        .collect::<Vec<_>>();

    receipt_dialog.set_items(ModelRc::new(slint::VecModel::from(items)));

    // Callback pour l'impression
    receipt_dialog.on_print_clicked({
        let receipt_weak = receipt_dialog.as_weak();
        move || {
            if let Some(rd) = receipt_weak.upgrade() {
                // la logique d'impression
                // Par exemple, envoyer vers une imprimante, exporter en PDF, etc.
                println!("Impression du ticket de caisse...");

                // Afficher un message de succès
                show_info_dialog("Impression", "Le ticket a été envoyé à l'imprimante");

                // Fermer le dialogue du reçu
                let _ = rd.hide();
            }
        }
    });

    // Callback pour fermer le dialogue
    let dialog_handle_cancel = receipt_dialog.as_weak();
    receipt_dialog.on_close_clicked(move || {
        if let Some(rd) = dialog_handle_cancel.upgrade() {
            let _ = rd.hide();
        }
    });

    // Afficher le dialogue du reçu
    let _ = receipt_dialog.run();
}

fn show_new_sale_dialog(main_ui: &ui::MainWindow, current_user_id: Uuid) {
    // Créer le dialogue
    let dialog = ui::NewSaleDialog::new().unwrap();
    let new_sale_state = Arc::new(Mutex::new(NewSaleState::new()));

    // Charger les produits disponibles
    match queries::get_available_products() {
        Ok(products) => {
            let product_ui_items: Vec<ui::ProductUI> = products
                .into_iter()
                .map(|p| ui::ProductUI {
                    id: p.id.to_string().into(),
                    name: p.name.into(),
                    stock: p.stock_in_sale_units.to_string().into(),
                    price_offers: format!("{:.0}", p.price_per_sale_unit).into(),
                    // packaging_description: p.packaging_description.into(),
                })
                .collect();

            dialog.set_available_products(ModelRc::new(slint::VecModel::from(product_ui_items)));
        }
        Err(e) => {
            show_error_dialog(
                "Erreur",
                &format!("Impossible de charger les produits: {}", e),
            );
            return;
        }
    }

    // Callback pour ajouter un article au panier
    dialog.on_add_to_cart({
        let state = new_sale_state.clone();
        let dialog_weak = dialog.as_weak();
        move |product_id_str, quantity| {
            if let Some(d) = dialog_weak.upgrade() {
                // Récupérer les détails du produit
                match queries::get_product_details(&product_id_str) {
                    Ok(product) => {
                        // Vérifier le stock disponible
                        if quantity <= product.stock_in_sale_units as i32 {
                            let mut state_guard = state.lock().unwrap();
                            state_guard.add_item(
                                product_id_str.to_string(),
                                product.name,
                                product.price_per_sale_unit,
                                quantity,
                                product.packaging_description,
                            );

                            // Mettre à jour l'UI
                            let cart_items = state_guard.to_cart_ui_items();
                            let total = format!("{:.0}", state_guard.total_amount);

                            drop(state_guard);

                            d.set_cart_items(ModelRc::new(slint::VecModel::from(cart_items)));
                            d.set_total_amount(total.into());
                            d.set_status_message("".into());
                        } else {
                            d.set_status_message(
                                format!(
                                    "Stock insuffisant ! Stock disponible: {}",
                                    product.stock_in_sale_units
                                )
                                .into(),
                            );
                        }
                    }
                    Err(e) => {
                        d.set_status_message(format!("Erreur lors de l'ajout: {}", e).into());
                    }
                }
            }
        }
    });

    // Callback pour retirer un article du panier
    dialog.on_remove_from_cart({
        let state = new_sale_state.clone();
        let dialog_weak = dialog.as_weak();
        move |product_id_str| {
            if let Some(d) = dialog_weak.upgrade() {
                let mut state_guard = state.lock().unwrap();
                state_guard.remove_item(&product_id_str);

                let cart_items = state_guard.to_cart_ui_items();
                let total = format!("{:.0}", state_guard.total_amount);

                drop(state_guard);

                d.set_cart_items(ModelRc::new(slint::VecModel::from(cart_items)));
                d.set_total_amount(total.into());
                d.set_status_message("".into());
            }
        }
    });

    // Callback pour mettre à jour la quantité
    dialog.on_update_cart_quantity({
        let state = new_sale_state.clone();
        let dialog_weak = dialog.as_weak();
        move |product_id_str, new_quantity| {
            if let Some(d) = dialog_weak.upgrade() {
                if new_quantity > 0 {
                    let mut state_guard = state.lock().unwrap();
                    state_guard.update_quantity(&product_id_str, new_quantity);

                    let cart_items = state_guard.to_cart_ui_items();
                    let total = format!("{:.0}", state_guard.total_amount);

                    drop(state_guard);

                    d.set_cart_items(ModelRc::new(slint::VecModel::from(cart_items)));
                    d.set_total_amount(total.into());
                    d.set_status_message("".into());
                }
            }
        }
    });

    // Callback pour vider le panier
    dialog.on_clear_cart({
        let state = new_sale_state.clone();
        let dialog_weak = dialog.as_weak();
        move || {
            if let Some(d) = dialog_weak.upgrade() {
                let mut state_guard = state.lock().unwrap();
                state_guard.clear_cart();

                drop(state_guard);

                d.set_cart_items(ModelRc::new(slint::VecModel::default()));
                d.set_total_amount("0".into());
                d.set_status_message("".into());
            }
        }
    });

    let details_dialog_weak = dialog.as_weak();
    dialog.on_cancel_clicked(move || {
        if let Some(dd) = details_dialog_weak.upgrade() {
            let _ = dd.hide();
        }
    });

    // Callback pour sauvegarder la vente
    dialog.on_save_clicked({
        let state = new_sale_state.clone();
        let dialog_weak = dialog.as_weak();
        let main_ui_weak = main_ui.as_weak();
        move || {
            if let Some(d) = dialog_weak.upgrade() {
                let state_guard = state.lock().unwrap();

                if state_guard.cart_items.is_empty() {
                    d.set_status_message("Veuillez ajouter au moins un article".into());
                    return;
                }

                // Préparer les données pour la sauvegarde
                let sale_data = CreateSaleData {
                    user_id: current_user_id,
                    items: state_guard
                        .cart_items
                        .values()
                        .map(|item| CreateSaleItemData {
                            product_id: Uuid::parse_str(&item.product_id).unwrap(),
                            quantity: item.quantity,
                        })
                        .collect(),
                };

                drop(state_guard);

                // Sauvegarder la vente
                match queries::create_sale(sale_data) {
                    Ok(receipt) => {
                        // Succès - fermer le dialogue et rafraîchir la liste
                        let _ = d.hide();

                        if let Some(main_ui) = main_ui_weak.upgrade() {
                            main_ui.invoke_request_sales();

                            // Rafraîchir automatiquement le dashboard
                            main_ui.invoke_refresh_dashboard();

                            // Rafraîchir automatiquement du rapport de vente
                            main_ui.invoke_request_report_data("7d".into());

                            // Afficher automatiquement le ticket de caisse
                            show_receipt_dialog(receipt);
                        }
                    }
                    Err(e) => {
                        d.set_status_message(format!("Erreur lors de la sauvegarde: {}", e).into());
                    }
                }
            }
        }
    });

    // Initialiser l'UI
    dialog.set_cart_items(ModelRc::new(slint::VecModel::default()));

    // Afficher le dialogue
    let _ = dialog.run();
}

/// Réinitialise les filtres
pub fn _reset_filters(sales_state: &Arc<Mutex<SalesState>>) {
    let mut state = sales_state.lock().unwrap();
    state.search_query.clear();
    state.date_filter = queries::DateFilter::All;
    state.sort_by = queries::SortFieldSale::Date;
    state.sort_order = queries::SortOrder::Desc;
    state.current_page = 1;
}
