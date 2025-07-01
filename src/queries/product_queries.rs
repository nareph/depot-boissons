// src/queries/product_queries.rs
use crate::{
    db,
    error::AppResult,
    models::{NewProduct, PackagingUnit, Product, ProductOffering},
    schema::{packaging_units, products},
};
use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ProductSearchParams {
    pub search_query: Option<String>,
    pub stock_filter: StockFilter,
    pub sort_by: SortField,
    pub sort_order: SortOrder,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone)]
pub enum StockFilter {
    All,
    InStock,    // stock > 0
    OutOfStock, // stock <= 0
}

#[derive(Debug, Clone)]
pub enum SortField {
    Name,
    Stock,
    CreatedAt,
}

#[derive(Debug, Clone)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug)]
pub struct PaginatedProducts {
    pub products: Vec<(Product, Vec<(ProductOffering, PackagingUnit)>)>,
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
            sort_by: SortField::Name,
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

    pub fn with_sort(mut self, field: SortField, order: SortOrder) -> Self {
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

/// Récupère tous les produits avec pagination, filtres et tri
pub fn get_products_paginated(params: ProductSearchParams) -> AppResult<PaginatedProducts> {
    let mut conn = db::get_conn()?;

    // Construction de la requête de base pour le comptage
    let mut count_query = products::table.into_boxed();

    // Application des filtres pour le comptage
    if let Some(search) = &params.search_query {
        let search_pattern = format!("%{}%", search.to_lowercase());
        count_query = count_query.filter(
            products::name
                .ilike(search_pattern.clone())
                .or(products::base_unit_name.ilike(search_pattern)),
        );
    }

    match params.stock_filter {
        StockFilter::All => {}
        StockFilter::InStock => {
            count_query = count_query.filter(products::total_stock_in_base_units.gt(0));
        }
        StockFilter::OutOfStock => {
            count_query = count_query.filter(products::total_stock_in_base_units.le(0));
        }
    }

    // Exécution du comptage
    let total_count = count_query.count().get_result::<i64>(&mut conn)?;

    // Construction de la requête pour les données
    let mut data_query = products::table.into_boxed();

    // Application des mêmes filtres pour les données
    if let Some(search) = &params.search_query {
        let search_pattern = format!("%{}%", search.to_lowercase());
        data_query = data_query.filter(
            products::name
                .ilike(search_pattern.clone())
                .or(products::base_unit_name.ilike(search_pattern)),
        );
    }

    match params.stock_filter {
        StockFilter::All => {}
        StockFilter::InStock => {
            data_query = data_query.filter(products::total_stock_in_base_units.gt(0));
        }
        StockFilter::OutOfStock => {
            data_query = data_query.filter(products::total_stock_in_base_units.le(0));
        }
    }

    // Application du tri
    match (params.sort_by, params.sort_order) {
        (SortField::Name, SortOrder::Asc) => {
            data_query = data_query.order(products::name.asc());
        }
        (SortField::Name, SortOrder::Desc) => {
            data_query = data_query.order(products::name.desc());
        }
        (SortField::Stock, SortOrder::Asc) => {
            data_query = data_query.order(products::total_stock_in_base_units.asc());
        }
        (SortField::Stock, SortOrder::Desc) => {
            data_query = data_query.order(products::total_stock_in_base_units.desc());
        }
        (SortField::CreatedAt, SortOrder::Asc) => {
            data_query = data_query.order(products::created_at.asc());
        }
        (SortField::CreatedAt, SortOrder::Desc) => {
            data_query = data_query.order(products::created_at.desc());
        }
    }

    // Application de la pagination
    let offset = (params.page - 1) * params.page_size;
    let products_page = data_query
        .limit(params.page_size)
        .offset(offset)
        .load::<Product>(&mut conn)?;

    // Chargement des offres pour les produits de cette page
    let all_offerings = ProductOffering::belonging_to(&products_page)
        .inner_join(packaging_units::table)
        .load::<(ProductOffering, PackagingUnit)>(&mut conn)?;

    let offerings_by_product = all_offerings.grouped_by(&products_page);

    let products_with_offers = products_page
        .into_iter()
        .zip(offerings_by_product)
        .collect::<Vec<_>>();

    let total_pages = (total_count + params.page_size - 1) / params.page_size;

    Ok(PaginatedProducts {
        products: products_with_offers,
        total_count,
        page: params.page,
        page_size: params.page_size,
        total_pages,
    })
}

/// Récupère tous les produits et leurs offres associées (version simple, conservée pour compatibilité)
pub fn get_all_products_with_offers()
-> AppResult<Vec<(Product, Vec<(ProductOffering, PackagingUnit)>)>> {
    let params = ProductSearchParams::new().with_pagination(1, 1000); // Grande page pour récupérer tout
    let result = get_products_paginated(params)?;
    Ok(result.products)
}

/// Recherche rapide de produits par nom
pub fn search_products_by_name(query: &str, limit: Option<i64>) -> AppResult<Vec<Product>> {
    use crate::schema::products::dsl::*;
    let mut conn = db::get_conn()?;

    let search_pattern = format!("%{}%", query.to_lowercase());
    let mut db_query = products
        .filter(name.ilike(&search_pattern))
        .order(name.asc())
        .into_boxed();

    if let Some(limit_val) = limit {
        db_query = db_query.limit(limit_val);
    }

    let results = db_query.load::<Product>(&mut conn)?;
    Ok(results)
}

/// Obtient les statistiques des produits
pub fn get_products_statistics() -> AppResult<ProductStatistics> {
    use crate::schema::products::dsl::*;
    let mut conn = db::get_conn()?;

    let total_products = products.count().get_result::<i64>(&mut conn)?;

    let in_stock_count = products
        .filter(total_stock_in_base_units.gt(0))
        .count()
        .get_result::<i64>(&mut conn)?;

    let out_of_stock_count = products
        .filter(total_stock_in_base_units.le(0))
        .count()
        .get_result::<i64>(&mut conn)?;

    let total_stock_value: Option<i64> = products
        .select(diesel::dsl::sql::<
            diesel::sql_types::Nullable<diesel::sql_types::BigInt>,
        >("SUM(total_stock_in_base_units)"))
        .first(&mut conn)?;

    Ok(ProductStatistics {
        total_products,
        in_stock_count,
        out_of_stock_count,
        total_stock_value: total_stock_value.unwrap_or(0),
    })
}

#[derive(Debug)]
pub struct ProductStatistics {
    pub total_products: i64,
    pub in_stock_count: i64,
    pub out_of_stock_count: i64,
    pub total_stock_value: i64,
}

/// Crée un nouveau produit
pub fn create_product(
    _name: &str,
    _base_unit_name: &str,
    initial_stock: i32,
) -> AppResult<Product> {
    use crate::schema::products::dsl::*;
    let mut conn = db::get_conn()?;

    let new_product = NewProduct {
        id: Uuid::new_v4(),
        name: _name,
        base_unit_name: _base_unit_name,
        total_stock_in_base_units: initial_stock,
    };

    let created_product = diesel::insert_into(products)
        .values(&new_product)
        .get_result(&mut conn)?;

    Ok(created_product)
}

/// Met à jour un produit existant
pub fn update_product(
    product_id: Uuid,
    new_name: &str,
    new_base_unit_name: &str,
    new_stock: i32,
) -> AppResult<Product> {
    use crate::schema::products::dsl::*;
    let mut conn = db::get_conn()?;

    let updated_product = diesel::update(products.find(product_id))
        .set((
            name.eq(new_name),
            base_unit_name.eq(new_base_unit_name),
            total_stock_in_base_units.eq(new_stock),
            updated_at.eq(Utc::now()),
        ))
        .get_result(&mut conn)?;

    Ok(updated_product)
}

/// Supprime un produit et toutes ses offres associées
pub fn delete_product(product_id: Uuid) -> AppResult<usize> {
    let mut conn = db::get_conn()?;

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        // Supprimer d'abord les offres
        use crate::schema::product_offerings::dsl as offerings_dsl;
        diesel::delete(
            offerings_dsl::product_offerings.filter(offerings_dsl::product_id.eq(product_id)),
        )
        .execute(conn)?;

        // Ensuite, supprimer le produit
        use crate::schema::products::dsl::*;
        let num_deleted = diesel::delete(products.filter(id.eq(product_id))).execute(conn)?;

        Ok(num_deleted)
    })
    .map_err(Into::into)
}

/// Récupère un produit par son ID
pub fn get_product_by_id(product_id: Uuid) -> AppResult<Product> {
    use crate::schema::products::dsl::*;
    let mut conn = db::get_conn()?;

    let product = products.find(product_id).first::<Product>(&mut conn)?;
    Ok(product)
}

/// Vérifie si un produit peut être supprimé (pas de ventes associées)
pub fn can_delete_product(product_id: Uuid) -> AppResult<bool> {
    use crate::schema::{product_offerings, sale_items};
    let mut conn = db::get_conn()?;

    // Vérifier s'il existe des ventes liées à ce produit
    let sales_count = sale_items::table
        .inner_join(product_offerings::table)
        .filter(product_offerings::product_id.eq(product_id))
        .count()
        .get_result::<i64>(&mut conn)?;

    Ok(sales_count == 0)
}
