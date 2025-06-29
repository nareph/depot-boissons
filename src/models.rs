// Fichier : src/models.rs

use crate::schema::{packaging_units, product_offerings, products, sale_items, sales, users};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

//================//
//    PRODUCTS    //
//================//
// Représente un produit de base, comme "Guinness". Le stock est en unité de base (bouteille/canette).
#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = products)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub base_unit_name: String,
    pub total_stock_in_base_units: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = products)]

pub struct NewProduct<'a> {
    pub id: Uuid,
    pub name: &'a str,
    pub base_unit_name: &'a str,
    pub total_stock_in_base_units: i32,
}

//=====================//
//  PACKAGING UNITS    //
//=====================//
// Représente un type de conditionnement, comme "Casier de 24".
#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = packaging_units)]
pub struct PackagingUnit {
    pub id: Uuid,
    pub name: String,
    pub contained_base_units: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = packaging_units)]
pub struct NewPackagingUnit<'a> {
    pub id: Uuid,
    pub name: &'a str,
    pub contained_base_units: i32,
}

//=======================//
//  PRODUCT OFFERINGS    //
//=======================//
// L'entité centrale : ce qui est réellement vendu. Lie un produit, un conditionnement et un prix.
#[derive(
    Queryable, Selectable, Identifiable, Associations, Serialize, Deserialize, Debug, Clone,
)]
#[diesel(belongs_to(Product))]
#[diesel(belongs_to(PackagingUnit))]
#[diesel(table_name = product_offerings)]
pub struct ProductOffering {
    pub id: Uuid,
    pub product_id: Uuid,
    pub packaging_unit_id: Uuid,
    pub price: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = product_offerings)]
pub struct NewProductOffering {
    pub id: Uuid,
    pub product_id: Uuid,
    pub packaging_unit_id: Uuid,
    pub price: BigDecimal,
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

#[derive(Insertable, Serialize, Deserialize, Debug)]
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
// Représente une ligne sur une facture. Pointe vers une "ProductOffering".
#[derive(
    Queryable, Selectable, Identifiable, Associations, Serialize, Deserialize, Debug, Clone,
)]
#[diesel(belongs_to(Sale))]
#[diesel(belongs_to(ProductOffering))]
#[diesel(table_name = sale_items)]
pub struct SaleItem {
    pub id: Uuid,
    pub sale_id: Uuid,
    pub product_offering_id: Uuid,
    pub quantity: i32,
    pub unit_price: BigDecimal,
    pub total_price: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = sale_items)]
pub struct NewSaleItem {
    pub id: Uuid,
    pub sale_id: Uuid,
    pub product_offering_id: Uuid,
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
    pub email: String,
    pub password: String,
    pub name: String,
    pub role: String,
    pub must_change_password: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub id: Uuid,
    pub email: &'a str,
    pub password: &'a str,
    pub name: &'a str,
    pub role: &'a str,
    pub must_change_password: bool,
}
