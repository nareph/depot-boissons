// src/queries/reporting_queries.rs

use crate::{db, error::AppResult, models::Product};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

// Structure pour contenir toutes les données agrégées pour un rapport
#[derive(Debug, Clone, Default)]
pub struct ReportData {
    pub total_revenue: BigDecimal,
    pub total_sales: i64,
    pub top_products: Vec<(Product, i64)>, // (Produit, Quantité totale vendue)
}

// Structure pour le résultat de la requête d'agrégation des produits
#[derive(Queryable, Debug, Clone)]
struct TopProductResult {
    product_id: Uuid,
    total_quantity: Option<i64>, // sum(Int4) -> BigInt (i64), et il peut être NULL
}

/// Génère des données de rapport pour une période donnée.
pub fn get_report_data(
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> AppResult<ReportData> {
    use crate::schema::{products, sale_items, sales};
    let mut conn = db::get_conn()?;

    // --- 1. KPI principaux (ne change pas) ---
    let (total_revenue, total_sales) = sales::table
        .filter(sales::date.between(start_date, end_date))
        .select((
            diesel::dsl::sum(sales::total_amount),
            diesel::dsl::count(sales::id),
        ))
        .first::<(Option<BigDecimal>, i64)>(&mut conn)?;

    // --- 2. Top 5 des produits vendus (par quantité) ---
    let top_products_query = sale_items::table
        .inner_join(sales::table.on(sale_items::sale_id.eq(sales::id)))
        .filter(sales::date.between(start_date, end_date))
        .group_by(sale_items::product_id)
        .select((
            sale_items::product_id,
            diesel::dsl::sum(sale_items::quantity),
        ))
        .order(diesel::dsl::sum(sale_items::quantity).desc().nulls_last()) // Trier avant de limiter
        .limit(5);

    // On charge le résultat dans notre struct explicite
    let top_product_results: Vec<TopProductResult> = top_products_query.load(&mut conn)?;

    // On convertit le résultat en une structure plus simple pour la suite
    let top_products_with_quantities: Vec<(Uuid, i64)> = top_product_results
        .into_iter()
        .map(|r| (r.product_id, r.total_quantity.unwrap_or(0)))
        .collect();

    let top_product_ids: Vec<Uuid> = top_products_with_quantities
        .iter()
        .map(|(id, _)| *id)
        .collect();

    if top_product_ids.is_empty() {
        return Ok(ReportData {
            total_revenue: total_revenue.unwrap_or_else(|| BigDecimal::from(0)),
            total_sales,
            top_products: vec![],
        });
    }

    let top_products_details: Vec<Product> = products::table
        .filter(products::id.eq_any(top_product_ids))
        .load(&mut conn)?;

    // On recrée une Map pour associer facilement, en respectant l'ordre du tri
    let details_map: std::collections::HashMap<Uuid, Product> = top_products_details
        .into_iter()
        .map(|p| (p.id, p))
        .collect();

    let final_top_products = top_products_with_quantities
        .into_iter()
        .filter_map(|(id, quantity)| details_map.get(&id).map(|p| (p.clone(), quantity)))
        .collect();

    Ok(ReportData {
        total_revenue: total_revenue.unwrap_or_else(|| BigDecimal::from(0)),
        total_sales,
        top_products: final_top_products,
    })
}
