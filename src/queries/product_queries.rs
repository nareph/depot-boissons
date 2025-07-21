// src/queries/product_queries.rs

use crate::{
    db,
    error::AppResult,
    models::{NewProduct, Product},
    schema,
};
use bigdecimal::BigDecimal;
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

use super::SortOrder;

#[derive(Debug, Clone)]
pub struct ProductSearchParams {
    pub search_query: Option<String>,
    pub stock_filter: StockFilter,
    pub sort_by: SortFieldProduct,
    pub sort_order: SortOrder,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Copy)]
pub enum StockFilter {
    All,
    InStock,    // stock > 0
    OutOfStock, // stock <= 0
}

#[derive(Debug, Clone, Copy)]
pub enum SortFieldProduct {
    Name,
    Stock,
    Price,
    CreatedAt,
}

#[derive(Debug)]
pub struct PaginatedProducts {
    pub products: Vec<Product>,
    pub total_count: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

impl Default for ProductSearchParams {
    fn default() -> Self {
        Self {
            search_query: None,
            stock_filter: StockFilter::All,
            sort_by: SortFieldProduct::Name,
            sort_order: SortOrder::Asc,
            page: 1,
            page_size: 10,
        }
    }
}

impl ProductSearchParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_search(mut self, query: Option<String>) -> Self {
        self.search_query = query.filter(|s| !s.trim().is_empty());
        self
    }

    pub fn with_stock_filter(mut self, filter: StockFilter) -> Self {
        self.stock_filter = filter;
        self
    }

    pub fn with_sort(mut self, field: SortFieldProduct, order: SortOrder) -> Self {
        self.sort_by = field;
        self.sort_order = order;
        self
    }

    pub fn with_pagination(mut self, page: i64, page_size: i64) -> Self {
        self.page = page.max(1);
        self.page_size = page_size.clamp(1, 100);
        self
    }
}

/// Récupère les produits avec pagination, filtres et tri, adapté au nouveau schéma.
pub fn get_products_paginated(params: ProductSearchParams) -> AppResult<PaginatedProducts> {
    use crate::schema::products::dsl::*;

    let mut conn = db::get_conn()?;

    // Construction de la requête de comptage
    let mut count_query = schema::products::table.into_boxed();

    // Application des filtres pour le comptage
    if let Some(search) = &params.search_query {
        let search_pattern = format!("%{}%", search.to_lowercase());
        count_query = count_query.filter(
            name.like(search_pattern.clone()) // SQLite utilise LIKE au lieu d'ILIKE
                .or(packaging_description.like(search_pattern)),
        );
    }

    match params.stock_filter {
        StockFilter::All => {}
        StockFilter::InStock => count_query = count_query.filter(stock_in_sale_units.gt(0)),
        StockFilter::OutOfStock => count_query = count_query.filter(stock_in_sale_units.le(0)),
    }

    // Comptage du total (avec les filtres appliqués)
    let total_count = count_query.count().get_result::<i64>(&mut conn)?;

    // Construction de la requête de données avec les mêmes filtres
    let mut data_query = schema::products::table.into_boxed();

    // Application des mêmes filtres pour les données
    if let Some(search) = &params.search_query {
        let search_pattern = format!("%{}%", search.to_lowercase());
        data_query = data_query.filter(
            name.like(search_pattern.clone())
                .or(packaging_description.like(search_pattern)),
        );
    }

    match params.stock_filter {
        StockFilter::All => {}
        StockFilter::InStock => data_query = data_query.filter(stock_in_sale_units.gt(0)),
        StockFilter::OutOfStock => data_query = data_query.filter(stock_in_sale_units.le(0)),
    }

    // Application du tri
    match (params.sort_by, params.sort_order) {
        (SortFieldProduct::Name, SortOrder::Asc) => data_query = data_query.order(name.asc()),
        (SortFieldProduct::Name, SortOrder::Desc) => data_query = data_query.order(name.desc()),
        (SortFieldProduct::Stock, SortOrder::Asc) => {
            data_query = data_query.order(stock_in_sale_units.asc())
        }
        (SortFieldProduct::Stock, SortOrder::Desc) => {
            data_query = data_query.order(stock_in_sale_units.desc())
        }
        (SortFieldProduct::Price, SortOrder::Asc) => {
            data_query = data_query.order(price_per_sale_unit.asc())
        }
        (SortFieldProduct::Price, SortOrder::Desc) => {
            data_query = data_query.order(price_per_sale_unit.desc())
        }
        (SortFieldProduct::CreatedAt, SortOrder::Asc) => {
            data_query = data_query.order(created_at.asc())
        }
        (SortFieldProduct::CreatedAt, SortOrder::Desc) => {
            data_query = data_query.order(created_at.desc())
        }
    }

    // Application de la pagination
    let offset = (params.page - 1) * params.page_size;
    let products_page = data_query
        .limit(params.page_size)
        .offset(offset)
        .load::<Product>(&mut conn)?;

    let total_pages = (total_count + params.page_size - 1) / params.page_size;

    Ok(PaginatedProducts {
        products: products_page,
        total_count,
        page: params.page,
        page_size: params.page_size,
        total_pages,
    })
}

/// Crée un nouveau produit fini (SKU).
pub fn create_product(
    p_name: String,
    p_packaging: String,
    p_stock: i32,
    p_price: BigDecimal,
) -> AppResult<Product> {
    use crate::schema::products::dsl::*;

    let mut conn = db::get_conn()?;

    let product_id = Uuid::new_v4().to_string(); // Convertir UUID en String
    let new_sku = generate_sku(&p_name, &p_packaging);
    let price_str = p_price.to_string(); // Convertir BigDecimal en String
    let now = Utc::now()
        .naive_utc()
        .format("%Y-%m-%d %H:%M:%S%.f")
        .to_string();

    let new_product = NewProduct {
        id: product_id.clone(),
        name: p_name,
        packaging_description: p_packaging,
        sku: Some(new_sku),
        stock_in_sale_units: p_stock,
        price_per_sale_unit: price_str,
    };

    diesel::insert_into(products)
        .values(&new_product)
        .execute(&mut conn)?;

    // Récupérer le produit après insertion
    products
        .find(product_id)
        .first::<Product>(&mut conn)
        .map_err(Into::into)
}

/// Met à jour un produit existant.
pub fn update_product(
    product_id: &str, // Changé de Uuid à &str
    new_name: String,
    new_packaging: String,
    new_stock: i32,
    new_price: BigDecimal,
) -> AppResult<Product> {
    use crate::schema::products::dsl::*;
    let mut conn = db::get_conn()?;

    let new_sku = generate_sku(&new_name, &new_packaging);
    let price_str = new_price.to_string(); // Convertir BigDecimal en String
    let now = Utc::now()
        .naive_utc()
        .format("%Y-%m-%d %H:%M:%S%.f")
        .to_string();

    diesel::update(products.find(product_id))
        .set((
            name.eq(&new_name),
            packaging_description.eq(&new_packaging),
            sku.eq(Some(&new_sku)),
            stock_in_sale_units.eq(new_stock),
            price_per_sale_unit.eq(&price_str),
            updated_at.eq(&now),
        ))
        .execute(&mut conn)?;

    // Récupérer le produit mis à jour
    products
        .find(product_id)
        .first::<Product>(&mut conn)
        .map_err(Into::into)
}

/// Supprime un produit. La suppression échouera si des ventes y sont liées (contrainte FK).
pub fn delete_product(product_id: &str) -> AppResult<usize> {
    // Changé de Uuid à &str
    use crate::schema::products::dsl::*;
    let mut conn = db::get_conn()?;
    diesel::delete(products.find(product_id))
        .execute(&mut conn)
        .map_err(Into::into)
}

/// Récupère un produit par son ID.
pub fn get_product_by_id(product_id: &str) -> AppResult<Product> {
    // Changé de Uuid à &str
    use crate::schema::products::dsl::*;
    let mut conn = db::get_conn()?;
    products
        .find(product_id)
        .first::<Product>(&mut conn)
        .map_err(Into::into)
}

/// Fonction d'aide pour générer un SKU standardisé.
fn generate_sku(product_name: &str, packaging: &str) -> String {
    let name_part = product_name
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(4)
        .collect::<String>()
        .to_uppercase();
    let packaging_part = packaging
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .collect::<String>()
        .to_uppercase();
    let unique_part = &Uuid::new_v4().to_string()[..6];
    format!("{}-{}-{}", name_part, packaging_part, unique_part)
}

/// Vérifie si un produit peut être supprimé (pas de ventes associées)
pub fn can_delete_product(p_id: &str) -> AppResult<bool> {
    // Changé de Uuid à &str
    use crate::schema::sale_items::dsl::*;
    let mut conn = db::get_conn()?;

    let sales_count = sale_items
        .filter(product_id.eq(p_id))
        .count()
        .get_result::<i64>(&mut conn)?;

    Ok(sales_count == 0)
}

pub fn get_available_products() -> AppResult<Vec<Product>> {
    use crate::db;
    use crate::schema::products;
    use diesel::prelude::*;

    let mut conn = db::get_conn()?;

    products::table
        .filter(products::stock_in_sale_units.gt(0)) // Seulement les produits en stock
        .order(products::name.asc())
        .load::<Product>(&mut conn)
        .map_err(Into::into)
}

/// Fonction pour récupérer les détails d'un produit spécifique
pub fn get_product_details(product_id_str: &str) -> AppResult<Product> {
    use crate::db;
    use crate::schema::products;
    use diesel::prelude::*;

    let mut conn = db::get_conn()?;

    // Pas besoin de parser l'UUID puisqu'on utilise maintenant des Strings
    products::table
        .find(product_id_str)
        .first::<Product>(&mut conn)
        .map_err(Into::into)
}

/// Met à jour le stock d'un produit (utile pour les ventes)
pub fn update_product_stock(product_id: &str, new_stock: i32) -> AppResult<()> {
    use crate::schema::products::dsl::*;
    let mut conn = db::get_conn()?;
    let now = Utc::now()
        .naive_utc()
        .format("%Y-%m-%d %H:%M:%S%.f")
        .to_string();

    diesel::update(products.find(product_id))
        .set((stock_in_sale_units.eq(new_stock), updated_at.eq(&now)))
        .execute(&mut conn)?;

    Ok(())
}

/// Diminue le stock d'un produit (pour les ventes)
pub fn decrease_product_stock(product_id: &str, quantity: i32) -> AppResult<()> {
    use crate::schema::products::dsl::*;
    let mut conn = db::get_conn()?;

    // Récupérer le produit actuel
    let product = products.find(product_id).first::<Product>(&mut conn)?;

    let new_stock = product.stock_in_sale_units - quantity;
    if new_stock < 0 {
        return Err(crate::error::AppError::ValidationError(
            "Stock insuffisant".to_string(),
        ));
    }

    update_product_stock(product_id, new_stock)
}
