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
// Représente un produit fini (SKU) tel qu'il est vendu, avec son stock et son prix.
#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = products)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub packaging_description: String,
    pub sku: Option<String>, // Le SKU est optionnel
    pub stock_in_sale_units: i32,
    pub price_per_sale_unit: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Représente les données nécessaires pour créer un nouveau produit fini.
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
// Représente une transaction globale (une facture).
#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = sales)]
pub struct Sale {
    pub id: Uuid,
    pub sale_number: String,
    pub total_amount: BigDecimal,
    pub date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = sales)]
pub struct NewSale<'a> {
    pub id: Uuid,
    pub sale_number: &'a str,
    pub total_amount: BigDecimal,
    pub date: DateTime<Utc>,
}

//================//
//  SALE ITEMS    //
//================//
// Représente une ligne sur une facture.
#[derive(
    Queryable, Selectable, Identifiable, Associations, Serialize, Deserialize, Debug, Clone,
)]
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

#[derive(Insertable, Debug)]
#[diesel(table_name = sale_items)]
pub struct NewSaleItem {
    pub id: Uuid,
    pub sale_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: BigDecimal,
    pub total_price: BigDecimal,
}

//===========//
//   USERS   //
//===========//
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

#[derive(Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub id: Uuid,
    pub password: &'a str,
    pub name: &'a str,
    pub role: &'a str,
    pub must_change_password: bool,
}
