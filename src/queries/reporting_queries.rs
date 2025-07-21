// src/queries/reporting_queries.rs

use crate::{db, error::AppResult, models::Product};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use std::collections::HashMap;

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
    product_id: String,          // Changé de Uuid à String
    total_quantity: Option<i64>, // sum(Int4) -> BigInt (i64), et il peut être NULL
}

/// Génère des données de rapport pour une période donnée.
pub fn get_report_data(
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> AppResult<ReportData> {
    use crate::schema::{products, sale_items, sales};
    let mut conn = db::get_conn()?;

    // Conversion des dates en String pour SQLite
    let start_date_str = start_date
        .naive_utc()
        .format("%Y-%m-%d %H:%M:%S%.f")
        .to_string();
    let end_date_str = end_date
        .naive_utc()
        .format("%Y-%m-%d %H:%M:%S%.f")
        .to_string();

    // --- 1. KPI principaux ---
    // Pour SQLite, nous devons récupérer les données et calculer manuellement
    let sales_in_period: Vec<(String, String)> = sales::table
        .filter(sales::date.between(&start_date_str, &end_date_str))
        .select((sales::total_amount, sales::id))
        .load(&mut conn)?;

    let total_sales = sales_in_period.len() as i64;
    let mut total_revenue = BigDecimal::from(0);

    // Calcul manuel du total
    for (amount_str, _) in &sales_in_period {
        match amount_str.parse::<BigDecimal>() {
            Ok(amount) => total_revenue += amount,
            Err(e) => {
                log::warn!("Erreur lors du parsing du montant '{}': {}", amount_str, e);
            }
        }
    }

    // --- 2. Top 5 des produits vendus (par quantité) ---
    let top_products_query = sale_items::table
        .inner_join(sales::table.on(sale_items::sale_id.eq(sales::id)))
        .filter(sales::date.between(&start_date_str, &end_date_str))
        .group_by(sale_items::product_id)
        .select((
            sale_items::product_id,
            diesel::dsl::sum(sale_items::quantity),
        ))
        .order(diesel::dsl::sum(sale_items::quantity).desc())
        .limit(5);

    // On charge le résultat dans notre struct explicite
    let top_product_results: Vec<TopProductResult> = top_products_query.load(&mut conn)?;

    // On convertit le résultat en une structure plus simple pour la suite
    let top_products_with_quantities: Vec<(String, i64)> = top_product_results
        .into_iter()
        .map(|r| (r.product_id, r.total_quantity.unwrap_or(0)))
        .collect();

    let top_product_ids: Vec<String> = top_products_with_quantities
        .iter()
        .map(|(id, _)| id.clone())
        .collect();

    if top_product_ids.is_empty() {
        return Ok(ReportData {
            total_revenue,
            total_sales,
            top_products: vec![],
        });
    }

    let top_products_details: Vec<Product> = products::table
        .filter(products::id.eq_any(top_product_ids))
        .load(&mut conn)?;

    // On recrée une Map pour associer facilement, en respectant l'ordre du tri
    let details_map: HashMap<String, Product> = top_products_details
        .into_iter()
        .map(|p| (p.id.clone(), p))
        .collect();

    let final_top_products = top_products_with_quantities
        .into_iter()
        .filter_map(|(id, quantity)| details_map.get(&id).map(|p| (p.clone(), quantity)))
        .collect();

    Ok(ReportData {
        total_revenue,
        total_sales,
        top_products: final_top_products,
    })
}
