// src/queries/sale_queries.rs
use super::SortOrder;
use crate::{
    db,
    error::AppResult,
    models::{
        CreateSaleData, NewSale, NewSaleItem, Product, Receipt, ReceiptItem, Sale, SaleItem,
        SaleWithItems,
    },
    schema::{products, sale_items, sales, users},
};
use bigdecimal::BigDecimal;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;

// Allow columns from sales and users tables to appear in the same GROUP BY clause
diesel::allow_columns_to_appear_in_same_group_by_clause!(
    sales::id,
    sales::user_id,
    sales::sale_number,
    sales::total_amount,
    sales::date,
    sales::created_at,
    sales::updated_at,
    users::name
);

// --- Structures pour la recherche et la pagination des ventes ---

#[derive(Debug, Clone)]
pub struct SaleSearchParams {
    /// Si Some(user_id), filtre par cet utilisateur. Si None, ne filtre pas (pour les admins).
    pub user_id_filter: Option<String>, // Changed from Uuid to String
    /// Filtre par numéro de vente ou nom d'utilisateur.
    pub search_query: Option<String>,
    pub date_filter: DateFilter,
    pub sort_by: SortFieldSale,
    pub sort_order: SortOrder,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DateFilter {
    All,
    Today,
    Week,
    Month,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortFieldSale {
    Date,
    Amount,
    SaleNumber,
}

#[derive(Debug)]
pub struct PaginatedSales {
    /// La liste des ventes pour la page actuelle, avec le nom du vendeur et le nombre d'articles.
    pub sales: Vec<SaleWithSeller>,
    pub total_count: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

/// Structure pour une vente avec informations du vendeur
#[derive(Debug, Clone)]
pub struct SaleWithSeller {
    pub sale: Sale,
    pub seller_name: String,
    pub items_count: i64,
}

impl Default for SaleSearchParams {
    fn default() -> Self {
        Self {
            user_id_filter: None,
            search_query: None,
            date_filter: DateFilter::All,
            sort_by: SortFieldSale::Date,
            sort_order: SortOrder::Desc,
            page: 1,
            page_size: 5,
        }
    }
}

// --- Helper functions for conversions ---

/// Converts BigDecimal to String for database storage
fn bigdecimal_to_string(value: &BigDecimal) -> String {
    value.to_string()
}

/// Converts NaiveDateTime to String for database storage
fn datetime_to_string(value: &NaiveDateTime) -> String {
    value.format("%Y-%m-%d %H:%M:%S%.f").to_string()
}

/// Generates a UUID as String
fn generate_uuid_string() -> String {
    uuid::Uuid::new_v4().to_string()
}

// --- Fonctions CRUD et de recherche ---

/// Crée une nouvelle vente à partir des données fournies,
/// calcule les totaux, vérifie les stocks et met à jour la base de données.
pub fn create_sale(data: CreateSaleData) -> AppResult<Receipt> {
    use crate::error::AppError;
    let mut conn = db::get_conn()?;

    conn.transaction(|conn| {
        // --- 1. Validation et calculs préliminaires ---
        let mut total_amount = BigDecimal::from(0);
        let mut validated_items = Vec::new();

        if data.items.is_empty() {
            return Err(AppError::ValidationError(
                "Aucun article dans la vente".to_string(),
            ));
        }

        for item_data in &data.items {
            let product: Product = products::table.find(&item_data.product_id).first(conn)?;

            // Parse price from String to BigDecimal
            let product_price = product.get_price_as_decimal().map_err(|e| {
                AppError::ValidationError(format!(
                    "Prix invalide pour le produit {}: {}",
                    product.name, e
                ))
            })?;

            if product.stock_in_sale_units < item_data.quantity {
                return Err(AppError::ValidationError(format!(
                    "Stock insuffisant pour le produit {} (ID: {})",
                    product.name, product.id
                )));
            }

            let quantity_bd = BigDecimal::from(item_data.quantity);
            let total_price = &product_price * &quantity_bd;
            total_amount += &total_price;

            validated_items.push((item_data.clone(), product, product_price, total_price));
        }

        // --- 2. Création de la vente principale ---
        let seller_name: String = users::table
            .find(&data.user_id)
            .select(users::name)
            .first(conn)?;

        let sale_id = generate_uuid_string();
        let current_time = datetime_to_string(&Utc::now().naive_utc());

        let new_sale = NewSale {
            id: sale_id,
            user_id: data.user_id,
            sale_number: generate_sale_number(),
            total_amount: bigdecimal_to_string(&total_amount),
            date: current_time,
        };


        diesel::insert_into(sales::table)
            .values(&new_sale)
            .execute(conn)?;
        
        let created_sale: Sale = sales::table
            .order(sales::id.desc())
            .first(conn)?;

        // --- 3. Insertion des articles de vente et mise à jour des stocks ---
        let mut receipt_items = Vec::new();
        for (item_data, product, unit_price, total_price) in validated_items {
            let sale_item_id = generate_uuid_string();
            let new_sale_item = NewSaleItem {
                id: sale_item_id,
                sale_id: created_sale.id.clone(),
                product_id: item_data.product_id.clone(),
                quantity: item_data.quantity,
                unit_price: bigdecimal_to_string(&unit_price),
                total_price: bigdecimal_to_string(&total_price),
            };

            diesel::insert_into(sale_items::table)
                .values(&new_sale_item)
                .execute(conn)?;

            diesel::update(products::table.find(item_data.product_id))
                .set(
                    products::stock_in_sale_units
                        .eq(product.stock_in_sale_units - item_data.quantity),
                )
                .execute(conn)?;

            receipt_items.push(ReceiptItem {
                product_name: product.name,
                packaging_description: product.packaging_description,
                quantity: new_sale_item.quantity,
                unit_price: unit_price.clone(),
                total_price: total_price.clone(),
            });
        }

        // --- 4. Génération du reçu ---
        let sale_date = created_sale
            .get_date_as_datetime()
            .map_err(|e| AppError::ValidationError(format!("Date invalide: {}", e)))?;

        Ok(Receipt {
            sale_number: created_sale.sale_number,
            date: sale_date.format("%d/%m/%Y %H:%M").to_string(),
            seller_name,
            items: receipt_items,
            total_amount,
        })
    })
    .map_err(|e| AppError::Database(Box::new(e)))
}

/// Récupère les ventes avec pagination, filtres et tri.
/// Si user_id_filter est Some(id), ne retourne que les ventes de cet utilisateur.
/// Si user_id_filter est None, retourne toutes les ventes (pour les admins).
pub fn get_sales_paginated(params: SaleSearchParams) -> AppResult<PaginatedSales> {
    use crate::schema::{sales, users};
    let mut conn = db::get_conn()?;

    // Prepare the search pattern outside the closure to avoid borrowing issues
    let search_pattern = params
        .search_query
        .as_ref()
        .map(|search| format!("%{}%", search));

    // --- Helper function to build the base query ---
    let build_base_query = || {
        let mut query = sales::table.inner_join(users::table).into_boxed();

        // Application du filtre utilisateur (le plus important pour les permissions)
        if let Some(user_id) = &params.user_id_filter {
            query = query.filter(sales::user_id.eq(user_id));
        }

        // Application des autres filtres
        if let Some(pattern) = &search_pattern {
            query = query.filter(
                sales::sale_number
                    .like(pattern) // SQLite uses LIKE instead of ILIKE
                    .or(users::name.like(pattern)),
            );
        }

        // Filtre par date - SQLite uses string comparison for dates
        match params.date_filter {
            DateFilter::Today => {
                let today = Utc::now().date_naive();
                let start = datetime_to_string(&today.and_hms_opt(0, 0, 0).unwrap());
                let end = datetime_to_string(&today.and_hms_opt(23, 59, 59).unwrap());
                query = query.filter(sales::date.between(start, end));
            }
            DateFilter::Week => {
                let week_ago =
                    datetime_to_string(&(Utc::now() - chrono::Duration::days(7)).naive_utc());
                query = query.filter(sales::date.ge(week_ago));
            }
            DateFilter::Month => {
                let month_ago =
                    datetime_to_string(&(Utc::now() - chrono::Duration::days(30)).naive_utc());
                query = query.filter(sales::date.ge(month_ago));
            }
            DateFilter::All => {}
        }

        query
    };

    // --- ÉTAPE 1: Obtenir le comptage total ---
    let total_count = build_base_query().count().get_result::<i64>(&mut conn)?;

    // --- ÉTAPE 2: Obtenir les IDs pour la page actuelle avec tri ---
    let offset = (params.page - 1) * params.page_size;

    let mut id_query = build_base_query().select(sales::id);

    // Application du tri
    match (params.sort_by, &params.sort_order) {
        (SortFieldSale::Date, SortOrder::Asc) => {
            id_query = id_query.order(sales::date.asc());
        }
        (SortFieldSale::Date, SortOrder::Desc) => {
            id_query = id_query.order(sales::date.desc());
        }
        (SortFieldSale::Amount, SortOrder::Asc) => {
            id_query = id_query.order(sales::total_amount.asc());
        }
        (SortFieldSale::Amount, SortOrder::Desc) => {
            id_query = id_query.order(sales::total_amount.desc());
        }
        (SortFieldSale::SaleNumber, SortOrder::Asc) => {
            id_query = id_query.order(sales::sale_number.asc());
        }
        (SortFieldSale::SaleNumber, SortOrder::Desc) => {
            id_query = id_query.order(sales::sale_number.desc());
        }
    }

    let sale_ids = id_query
        .limit(params.page_size)
        .offset(offset)
        .load::<String>(&mut conn)?; // Changed from Uuid to String

    // Si la page est vide, retourner un résultat vide
    if sale_ids.is_empty() {
        return Ok(PaginatedSales {
            sales: vec![],
            total_count,
            page: params.page,
            page_size: params.page_size,
            total_pages: if total_count == 0 {
                1
            } else {
                (total_count + params.page_size - 1) / params.page_size
            },
        });
    }

    // --- ÉTAPE 3: Récupérer les données complètes ---
    // First, get the sales with seller names
    let mut sales_with_sellers_query = sales::table
        .inner_join(users::table)
        .filter(sales::id.eq_any(&sale_ids))
        .select((Sale::as_select(), users::name))
        .into_boxed();

    // Application du tri pour maintenir l'ordre
    match (params.sort_by, params.sort_order) {
        (SortFieldSale::Date, SortOrder::Asc) => {
            sales_with_sellers_query = sales_with_sellers_query.order(sales::date.asc());
        }
        (SortFieldSale::Date, SortOrder::Desc) => {
            sales_with_sellers_query = sales_with_sellers_query.order(sales::date.desc());
        }
        (SortFieldSale::Amount, SortOrder::Asc) => {
            sales_with_sellers_query = sales_with_sellers_query.order(sales::total_amount.asc());
        }
        (SortFieldSale::Amount, SortOrder::Desc) => {
            sales_with_sellers_query = sales_with_sellers_query.order(sales::total_amount.desc());
        }
        (SortFieldSale::SaleNumber, SortOrder::Asc) => {
            sales_with_sellers_query = sales_with_sellers_query.order(sales::sale_number.asc());
        }
        (SortFieldSale::SaleNumber, SortOrder::Desc) => {
            sales_with_sellers_query = sales_with_sellers_query.order(sales::sale_number.desc());
        }
    }

    let sales_with_sellers = sales_with_sellers_query.load::<(Sale, String)>(&mut conn)?;

    // Then, get the item counts for each sale
    use crate::schema::sale_items;
    let item_counts = sale_items::table
        .filter(sale_items::sale_id.eq_any(&sale_ids))
        .group_by(sale_items::sale_id)
        .select((sale_items::sale_id, diesel::dsl::count(sale_items::id)))
        .load::<(String, i64)>(&mut conn)?; // Changed from Uuid to String

    // Create a map for quick lookup of item counts
    let item_count_map: std::collections::HashMap<String, i64> = item_counts.into_iter().collect();

    // Transformer en structure plus claire
    let sales_with_seller: Vec<SaleWithSeller> = sales_with_sellers
        .into_iter()
        .map(|(sale, seller_name)| {
            let items_count = item_count_map.get(&sale.id).copied().unwrap_or(0);
            SaleWithSeller {
                sale,
                seller_name,
                items_count,
            }
        })
        .collect();

    let total_pages = if total_count == 0 {
        1
    } else {
        (total_count + params.page_size - 1) / params.page_size
    };

    Ok(PaginatedSales {
        sales: sales_with_seller,
        total_count,
        page: params.page,
        page_size: params.page_size,
        total_pages,
    })
}

/// Récupère une vente spécifique avec ses articles et informations associées.
/// Inclut une vérification des permissions : seul l'admin ou le propriétaire de la vente peut la voir.
pub fn get_sale_details(
    sale_id: &str,         // Changed from Uuid to &str
    current_user_id: &str, // Changed from Uuid to &str
    is_admin: bool,
) -> AppResult<SaleWithItems> {
    let mut conn = db::get_conn()?;

    let (sale, seller_name): (Sale, String) = sales::table
        .inner_join(users::table)
        .filter(sales::id.eq(sale_id))
        .select((Sale::as_select(), users::name))
        .first(&mut conn)?;

    // Vérification des permissions
    if !is_admin && sale.user_id != current_user_id {
        return Err(crate::error::AppError::Unauthorized(
            "Vous n'avez pas l'autorisation de voir cette vente".to_string(),
        ));
    }

    let items_with_products: Vec<(SaleItem, Product)> = SaleItem::belonging_to(&sale)
        .inner_join(products::table)
        .load(&mut conn)?;

    Ok(SaleWithItems {
        sale,
        items: items_with_products,
        seller_name,
    })
}

/// Génère un numéro de vente unique.
pub fn generate_sale_number() -> String {
    format!("VTE-{}", Utc::now().format("%Y%m%d%H%M%S"))
}

/// Fonction utilitaire pour créer les paramètres de recherche selon les permissions utilisateur
pub fn create_search_params_for_user(
    base_params: SaleSearchParams,
    current_user_id: String, // Changed from Uuid to String
    is_admin: bool,
) -> SaleSearchParams {
    SaleSearchParams {
        user_id_filter: if is_admin {
            None
        } else {
            Some(current_user_id)
        },
        ..base_params
    }
}

/// Génère un reçu/ticket de caisse pour une vente donnée
pub fn generate_receipt(sale_id: &str) -> AppResult<Receipt> {
    // Changed from Uuid to &str
    let mut conn = db::get_conn()?;

    // 1. Récupérer les informations de base de la vente et du vendeur
    let (sale, seller_name): (Sale, String) = sales::table
        .inner_join(users::table)
        .filter(sales::id.eq(sale_id))
        .select((sales::all_columns, users::name))
        .first(&mut conn)?;

    // 2. Récupérer tous les articles de la vente avec les infos produits
    let items_with_products: Vec<(SaleItem, Product)> = SaleItem::belonging_to(&sale)
        .inner_join(products::table)
        .load(&mut conn)?;

    // 3. Transformer les données en format Receipt
    let mut receipt_items = Vec::new();
    for (item, product) in items_with_products {
        let unit_price = item.get_unit_price_as_decimal().map_err(|e| {
            crate::error::AppError::ValidationError(format!("Prix unitaire invalide: {}", e))
        })?;
        let total_price = item.get_total_price_as_decimal().map_err(|e| {
            crate::error::AppError::ValidationError(format!("Prix total invalide: {}", e))
        })?;

        receipt_items.push(ReceiptItem {
            product_name: product.name,
            packaging_description: product.packaging_description,
            quantity: item.quantity,
            unit_price,
            total_price,
        });
    }

    // 4. Construire le reçu final
    let total_amount = sale.get_total_amount_as_decimal().map_err(|e| {
        crate::error::AppError::ValidationError(format!("Montant total invalide: {}", e))
    })?;

    let sale_date = sale
        .get_date_as_datetime()
        .map_err(|e| crate::error::AppError::ValidationError(format!("Date invalide: {}", e)))?;

    Ok(Receipt {
        sale_number: sale.sale_number,
        date: sale_date.format("%d/%m/%Y %H:%M").to_string(),
        seller_name,
        items: receipt_items,
        total_amount,
    })
}
