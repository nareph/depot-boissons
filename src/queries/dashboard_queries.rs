// src/queries/dashboard_queries.rs
use crate::{db, error::AppResult, models::Product};
use bigdecimal::BigDecimal;
use chrono::Utc;
use diesel::{
    dsl::{count, sum},
    prelude::*,
};

/// Calcule le chiffre d'affaires total et le nombre de ventes pour la journée en cours.
pub fn get_today_sales_summary() -> AppResult<(BigDecimal, i64)> {
    use crate::schema::sales::dsl::*;
    let mut conn = db::get_conn()?;

    let now_utc = Utc::now();
    let today_utc_naive = now_utc.date_naive();
    let start_of_day_utc = today_utc_naive.and_hms_opt(0, 0, 0).unwrap();
    let end_of_day_utc = today_utc_naive.and_hms_opt(23, 59, 59).unwrap();

    // Formatage des dates pour SQLite (String)
    let start_of_day_str = start_of_day_utc.format("%Y-%m-%d %H:%M:%S%.f").to_string();
    let end_of_day_str = end_of_day_utc.format("%Y-%m-%d %H:%M:%S%.f").to_string();

    log::info!(
        "Calcul du résumé des ventes entre {} et {}",
        start_of_day_str,
        end_of_day_str
    );

    // Pour SQLite, nous devons faire la somme et conversion manuellement
    // car total_amount est maintenant un String
    let sales_data = sales
        .filter(date.between(&start_of_day_str, &end_of_day_str))
        .select((total_amount, id))
        .load::<(String, String)>(&mut conn)?;

    let mut total_revenue = BigDecimal::from(0);
    let sales_count = sales_data.len() as i64;

    // Conversion manuelle et somme des montants
    for (amount_str, _) in sales_data {
        match amount_str.parse::<BigDecimal>() {
            Ok(amount) => total_revenue += amount,
            Err(e) => {
                log::warn!("Erreur lors du parsing du montant '{}': {}", amount_str, e);
            }
        }
    }

    Ok((total_revenue, sales_count))
}

/// Récupère la liste des produits dont le stock est inférieur ou égal à un certain seuil.
pub fn get_low_stock_products(threshold: i32) -> AppResult<Vec<Product>> {
    use crate::schema::products::dsl::*;
    let mut conn = db::get_conn()?;

    let low_stock_items = products
        .filter(stock_in_sale_units.le(threshold))
        .order(stock_in_sale_units.asc())
        .load::<Product>(&mut conn)?;

    Ok(low_stock_items)
}