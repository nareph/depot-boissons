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

/// Récupère tous les produits et leurs offres de prix associées.
pub fn get_all_products_with_offers()
-> AppResult<Vec<(Product, Vec<(ProductOffering, PackagingUnit)>)>> {
    let mut conn = db::get_conn()?;

    let all_products = products::table.load::<Product>(&mut conn)?;
    let all_offerings = ProductOffering::belonging_to(&all_products)
        .inner_join(packaging_units::table)
        .load::<(ProductOffering, PackagingUnit)>(&mut conn)?;

    let offerings_by_product = all_offerings.grouped_by(&all_products);

    let result = all_products
        .into_iter()
        .zip(offerings_by_product)
        .collect::<Vec<_>>();

    Ok(result)
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
        // On filtre la table `products` avant de la passer à `delete`
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
