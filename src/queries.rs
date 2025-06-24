// src/queries.rs

use crate::db;
use crate::error::AppResult;
use crate::models::{NewUser, PackagingUnit, Product, ProductOffering, User};
use crate::schema::{packaging_units, products};
use bigdecimal::BigDecimal;
use chrono::Local;
use diesel::dsl::{count, sum};
use diesel::prelude::*;
use uuid::Uuid;

//======================//
//  Fonctions existante //
//======================//

/// Met à jour le mot de passe d'un utilisateur et désactive le flag `must_change_password`.
pub fn update_user_password(user_id: Uuid, new_password_hash: &str) -> AppResult<()> {
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;

    diesel::update(users.find(user_id))
        .set((
            password.eq(new_password_hash),
            must_change_password.eq(false),
        ))
        .execute(&mut conn)?;

    Ok(())
}
// src/queries.rs

// ... (ajoutez cette fonction à votre fichier existant) ...

/// Récupère un utilisateur par son ID.
pub fn get_user_by_id(user_id: Uuid) -> AppResult<User> {
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;
    let user = users.find(user_id).first::<User>(&mut conn)?;
    Ok(user)
}
//==============================//
//  Fonctions pour le Dashboard //
//==============================//

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

//=====================================//
//  Fonctions pour la Gestion Produits //
//=====================================//

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

//=========================================//
//  Fonctions pour la Gestion Utilisateurs //
//=========================================//

/// Récupère tous les utilisateurs sauf celui qui est spécifié (pour ne pas que l'admin se supprime lui-même).
pub fn get_all_users(except_user_id: Uuid) -> AppResult<Vec<User>> {
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;
    let all_users = users
        .filter(id.ne(except_user_id))
        .order(name.asc())
        .load::<User>(&mut conn)?;
    Ok(all_users)
}

/// Crée un nouvel utilisateur. Le mot de passe doit déjà être haché.
pub fn create_user(
    new_name: &str,
    new_email: &str,
    hashed_password: &str,
    new_role: &str,
) -> AppResult<User> {
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;

    let new_user = NewUser {
        id: Uuid::new_v4(),
        name: new_name,
        email: new_email,
        password: hashed_password,
        role: new_role,
        must_change_password: true,
    };

    let created_user = diesel::insert_into(users)
        .values(&new_user)
        .get_result(&mut conn)?;

    Ok(created_user)
}

/// Supprime un utilisateur par son ID.
pub fn delete_user(user_id_to_delete: Uuid) -> AppResult<usize> {
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;
    let num_deleted = diesel::delete(users.find(user_id_to_delete)).execute(&mut conn)?;
    Ok(num_deleted)
}
