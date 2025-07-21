// src/models.rs

use crate::schema::{products, sale_items, sales, users};
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

//================//
//    PRODUCTS    //
//================//
#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = products)]
pub struct Product {
    pub id: String, // Changed from Uuid to String
    pub name: String,
    pub packaging_description: String,
    pub sku: Option<String>,
    pub stock_in_sale_units: i32,
    pub price_per_sale_unit: String, // Changed from BigDecimal to String
    pub created_at: String,          // Changed from NaiveDateTime to String
    pub updated_at: String,          // Changed from NaiveDateTime to String
}

#[derive(Insertable, Debug)]
#[diesel(table_name = products)]
pub struct NewProduct {
    pub id: String,
    pub name: String,
    pub packaging_description: String,
    pub sku: Option<String>,
    pub stock_in_sale_units: i32,
    pub price_per_sale_unit: String,
}

//============//
//   SALES    //
//============//
#[derive(Queryable, Selectable, Identifiable, Associations, Debug, Clone)]
#[diesel(belongs_to(User))]
#[diesel(table_name = sales)]
pub struct Sale {
    pub id: String,      // Changed from Uuid to String
    pub user_id: String, // Changed from Uuid to String
    pub sale_number: String,
    pub total_amount: String, // Changed from BigDecimal to String
    pub date: String,         // Changed from NaiveDateTime to String
    pub created_at: String,   // Changed from NaiveDateTime to String
    pub updated_at: String,   // Changed from NaiveDateTime to String
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = sales)]
pub struct NewSale {
    pub id: String,
    pub user_id: String,
    pub sale_number: String,
    pub total_amount: String,
    pub date: String,
}
//================//
//  SALE ITEMS    //
//================//
#[derive(Queryable, Selectable, Identifiable, Associations, Debug, Clone)]
#[diesel(belongs_to(Sale))]
#[diesel(belongs_to(Product))]
#[diesel(table_name = sale_items)]
pub struct SaleItem {
    pub id: String,         // Changed from Uuid to String
    pub sale_id: String,    // Changed from Uuid to String
    pub product_id: String, // Changed from Uuid to String
    pub quantity: i32,
    pub unit_price: String,  // Changed from BigDecimal to String
    pub total_price: String, // Changed from BigDecimal to String
    pub created_at: String,  // Changed from NaiveDateTime to String
    pub updated_at: String,  // Changed from NaiveDateTime to String
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = sale_items)]
pub struct NewSaleItem {
    pub id: String,
    pub sale_id: String,
    pub product_id: String,
    pub quantity: i32,
    pub unit_price: String,
    pub total_price: String,
}

//==============================//
//  Structs Composites          //
//==============================//
#[derive(Debug, Clone)]
pub struct SaleWithItems {
    pub sale: Sale,
    pub items: Vec<(SaleItem, Product)>,
    pub seller_name: String,
}

#[derive(Debug, Clone)]
pub struct CreateSaleData {
    pub user_id: String, // Changed from Uuid to String
    pub items: Vec<CreateSaleItemData>,
}

#[derive(Debug, Clone)]
pub struct CreateSaleItemData {
    pub product_id: String, // Changed from Uuid to String
    pub quantity: i32,
}

//==============================//
//  Structs pour Reçus          //
//==============================//
#[derive(Debug, Serialize)]
pub struct Receipt {
    pub sale_number: String,
    pub date: String,
    pub seller_name: String,
    pub items: Vec<ReceiptItem>,
    pub total_amount: BigDecimal, // Keep BigDecimal for business logic
}

#[derive(Debug, Serialize)]
pub struct ReceiptItem {
    pub product_name: String,
    pub packaging_description: String,
    pub quantity: i32,
    pub unit_price: BigDecimal,  // Keep BigDecimal for business logic
    pub total_price: BigDecimal, // Keep BigDecimal for business logic
}

//===========//
//   USERS   //
//===========//
#[derive(Queryable, Selectable, Identifiable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = users)]
pub struct User {
    pub id: String, // Changed from Uuid to String
    pub password: String,
    pub name: String,
    pub role: String,
    pub must_change_password: i32, // Changed from bool to i32 (SQLite uses integers for booleans)
    pub created_at: String,        // Changed from NaiveDateTime to String
    pub updated_at: String,        // Changed from NaiveDateTime to String
}

#[derive(Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: String,
    pub password: String,
    pub name: String,
    pub role: String,
    pub must_change_password: i32,
}

//==============================//
//  Helper functions for conversion
//==============================//
impl Product {
    /// Converts price from String to BigDecimal
    pub fn get_price_as_decimal(&self) -> Result<BigDecimal, bigdecimal::ParseBigDecimalError> {
        self.price_per_sale_unit.parse()
    }

    /// Converts created_at from String to NaiveDateTime
    pub fn get_created_at_as_datetime(&self) -> Result<NaiveDateTime, chrono::ParseError> {
        NaiveDateTime::parse_from_str(&self.created_at, "%Y-%m-%d %H:%M:%S%.f")
    }
}

impl Sale {
    /// Converts total_amount from String to BigDecimal
    pub fn get_total_amount_as_decimal(
        &self,
    ) -> Result<BigDecimal, bigdecimal::ParseBigDecimalError> {
        self.total_amount.parse()
    }

    /// Converts date from String to NaiveDateTime
    pub fn get_date_as_datetime(&self) -> Result<NaiveDateTime, chrono::ParseError> {
        NaiveDateTime::parse_from_str(&self.date, "%Y-%m-%d %H:%M:%S%.f")
    }
}

impl SaleItem {
    /// Converts unit_price from String to BigDecimal
    pub fn get_unit_price_as_decimal(
        &self,
    ) -> Result<BigDecimal, bigdecimal::ParseBigDecimalError> {
        self.unit_price.parse()
    }

    /// Converts total_price from String to BigDecimal
    pub fn get_total_price_as_decimal(
        &self,
    ) -> Result<BigDecimal, bigdecimal::ParseBigDecimalError> {
        self.total_price.parse()
    }
}

impl User {
    /// Converts must_change_password from i32 to bool
    pub fn must_change_password_as_bool(&self) -> bool {
        self.must_change_password != 0
    }
}
