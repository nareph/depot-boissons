// src/queries/dashboard_queries.rs
use crate::{db, error::AppResult, models::Product};
use bigdecimal::BigDecimal;
use chrono::Local;
use diesel::{
    dsl::{count, sum},
    prelude::*,
};

/// Calcule le chiffre d'affaires total et le nombre de ventes pour la journée en cours.
pub fn get_today_sales_summary() -> AppResult<(BigDecimal, i64)> {
    use crate::schema::sales::dsl::*;
    let mut conn = db::get_conn()?;

    let today = Local::now().date_naive();
    let start_of_day = today.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end_of_day = today.and_hms_opt(23, 59, 59).unwrap().and_utc();

    log::info!(
        "Calcul du résumé des ventes entre {} et {}",
        start_of_day,
        end_of_day
    );

    let summary = sales
        .filter(date.between(start_of_day, end_of_day))
        .select((sum(total_amount), count(id)))
        .first::<(Option<BigDecimal>, i64)>(&mut conn)?;

    let total_revenue = summary.0.unwrap_or_else(|| BigDecimal::from(0));
    let sales_count = summary.1;

    Ok((total_revenue, sales_count))
}

/// Récupère la liste des produits dont le stock est inférieur ou égal à un certain seuil.
pub fn get_low_stock_products(threshold: i32) -> AppResult<Vec<Product>> {
    use crate::schema::products::dsl::*;
    let mut conn = db::get_conn()?;

    let low_stock_items = products
        .filter(total_stock_in_base_units.le(threshold))
        .order(total_stock_in_base_units.asc())
        .load::<Product>(&mut conn)?;

    Ok(low_stock_items)
}
