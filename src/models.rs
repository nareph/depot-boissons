// src/models.rs

use crate::schema::{products, sale_items, sales, users};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

//================//
//    PRODUCTS    //
//================//
// Représente un produit fini (SKU) tel qu'il est vendu.
#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = products)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub packaging_description: String,
    pub sku: Option<String>,
    pub stock_in_sale_units: i32,
    pub price_per_sale_unit: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Pour insérer un nouveau produit.
#[derive(Insertable, Debug)]
#[diesel(table_name = products)]
pub struct NewProduct {
    pub id: Uuid,
    pub name: String,
    pub packaging_description: String,
    pub sku: Option<String>,
    pub stock_in_sale_units: i32,
    pub price_per_sale_unit: BigDecimal,
}

//============//
//   SALES    //
//============//
// Représente une transaction/facture dans la base de données.
#[derive(Queryable, Selectable, Identifiable, Associations, Debug, Clone)]
#[diesel(belongs_to(User))]
#[diesel(table_name = sales)]
pub struct Sale {
    pub id: Uuid,
    pub user_id: Uuid,
    pub sale_number: String,
    pub total_amount: BigDecimal,
    pub date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Pour insérer une nouvelle vente.
#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = sales)]
pub struct NewSale {
    pub id: Uuid,
    pub user_id: Uuid,
    pub sale_number: String,
    pub total_amount: BigDecimal,
    pub date: DateTime<Utc>,
}

//================//
//  SALE ITEMS    //
//================//
// Représente une ligne sur une facture.
#[derive(Queryable, Selectable, Identifiable, Associations, Debug, Clone)]
#[diesel(belongs_to(Sale))]
#[diesel(belongs_to(Product))]
#[diesel(table_name = sale_items)]
pub struct SaleItem {
    pub id: Uuid,
    pub sale_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: BigDecimal,
    pub total_price: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Pour insérer un nouvel article de vente.
#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = sale_items)]
pub struct NewSaleItem {
    pub id: Uuid,
    pub sale_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: BigDecimal,
    pub total_price: BigDecimal,
}

//==============================//
//  Structs Composites pour Ventes //
//==============================//

/// Structure pour représenter une vente complète avec ses articles et informations produit.
/// Utile pour afficher les détails d'une vente.
#[derive(Debug, Clone)]
pub struct SaleWithItems {
    pub sale: Sale,
    pub items: Vec<(SaleItem, Product)>,
    pub seller_name: String,
}

/// Contient toutes les informations nécessaires pour créer une nouvelle vente en base de données.
/// C'est l'objet qui sera passé à la fonction de requête.
#[derive(Debug, Clone)]
pub struct CreateSaleData {
    pub user_id: Uuid,
    pub items: Vec<CreateSaleItemData>,
}

/// Représente un article à insérer dans le cadre d'une nouvelle vente.
#[derive(Debug, Clone)]
pub struct CreateSaleItemData {
    pub product_id: Uuid,
    pub quantity: i32,
}

//==============================//
//  Structs pour Reçus/Affichage //
//==============================//

/// Structure de données pour générer un ticket de caisse.
#[derive(Debug, Serialize)]
pub struct Receipt {
    pub sale_number: String,
    pub date: String,
    pub seller_name: String,
    pub items: Vec<ReceiptItem>,
    pub total_amount: BigDecimal,
}

/// Représente un article sur le ticket de caisse.
#[derive(Debug, Serialize)]
pub struct ReceiptItem {
    pub product_name: String,
    pub packaging_description: String,
    pub quantity: i32,
    pub unit_price: BigDecimal,
    pub total_price: BigDecimal,
}

//===========//
//   USERS   //
//===========//
// Représente un utilisateur de l'application.
#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub password: String,
    pub name: String,
    pub role: String,
    pub must_change_password: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Pour insérer un nouvel utilisateur.
#[derive(Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub id: Uuid,
    pub password: &'a str,
    pub name: &'a str,
    pub role: &'a str,
    pub must_change_password: bool,
}
