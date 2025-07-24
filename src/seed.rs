// src/seed.rs

use crate::{
    error::AppResult,
    models::{NewProduct, NewSale, NewSaleItem, NewUser, Product},
};
use bcrypt::{DEFAULT_COST, hash};
use bigdecimal::BigDecimal;
use chrono::Utc;
use diesel::{prelude::*, sqlite::SqliteConnection};
use rand::Rng;
use uuid::Uuid;

pub fn seed_database(conn: &mut SqliteConnection) -> AppResult<()> {
    use crate::schema::{products, sale_items, sales, users};

    log::info!("--- Début du seeding de la base de données SQLite ---");

    diesel::delete(sale_items::table).execute(conn)?;
    diesel::delete(sales::table).execute(conn)?;
    diesel::delete(products::table).execute(conn)?;
    log::info!("Tables nettoyées.");

    log::info!("Récupérer l'ID d'un admin existant...");
    let admin_id = users::table
        .filter(users::name.eq("admin"))
        .select(users::id)
        .first::<String>(conn)?;

    log::info!("Création des produits (SKUs)...");

    // Conversion des BigDecimals en Strings pour SQLite
    let price1_str = "8500.00".to_string();
    let price2_str = "8500.00".to_string();
    let price3_str = "14500.00".to_string();
    let price4_str = "2500.00".to_string();
    let price5_str = "11000.00".to_string();

    let products_data = vec![
        NewProduct {
            id: Uuid::new_v4().to_string(),
            name: "Castel Beer".to_string(),
            packaging_description: "Casier 65cl de 12".to_string(),
            sku: Some("CAS-65-CAS12".to_string()),
            stock_in_sale_units: 100,
            price_per_sale_unit: price1_str,
        },
        NewProduct {
            id: Uuid::new_v4().to_string(),
            name: "33 Export".to_string(),
            packaging_description: "Casier 65cl de 12".to_string(),
            sku: Some("33EXP-65-CAS12".to_string()),
            stock_in_sale_units: 150,
            price_per_sale_unit: price2_str,
        },
        NewProduct {
            id: Uuid::new_v4().to_string(),
            name: "Guinness Smooth".to_string(),
            packaging_description: "Casier 33cl de 24".to_string(),
            sku: Some("GUIN-33-CAS24".to_string()),
            stock_in_sale_units: 80,
            price_per_sale_unit: price3_str,
        },
        NewProduct {
            id: Uuid::new_v4().to_string(),
            name: "Supermont Eau".to_string(),
            packaging_description: "Palette 1.5L de 12".to_string(),
            sku: Some("SPMT-1.5-PAL12".to_string()),
            stock_in_sale_units: 50,
            price_per_sale_unit: price4_str,
        },
        NewProduct {
            id: Uuid::new_v4().to_string(),
            name: "Coca-Cola".to_string(),
            packaging_description: "Casier 33cl de 24".to_string(),
            sku: Some("COKE-33-CAS24".to_string()),
            stock_in_sale_units: 45,
            price_per_sale_unit: price5_str,
        },
    ];

    diesel::insert_into(products::table)
        .values(&products_data)
        .execute(conn)?;
    let inserted_products = products::table.load::<Product>(conn)?;
    log::info!("Produits créés.");

    log::info!("Création des ventes de test pour aujourd'hui...");
    let mut rng = rand::rng();

    for i in 1..=5 {
        let sale_id_gen = Uuid::new_v4();
        let sale_id_str = sale_id_gen.to_string();
        let sale_number_gen = format!("VTE-{:05}", i);
        let date_gen = Utc::now().naive_utc();
        let date_str = date_gen.format("%Y-%m-%d %H:%M:%S%.f").to_string();
        let mut sale_total = BigDecimal::from(0);

        let num_items = rng.random_range(1..=3);
        let mut items_to_insert = Vec::new();

        for _ in 0..num_items {
            let product_to_sell = &inserted_products[rng.random_range(0..inserted_products.len())];
            let quantity_sold = rng.random_range(1..=5);

            let product_price = product_to_sell.get_price_as_decimal()?;
            let total_price = &product_price * BigDecimal::from(quantity_sold);

            // Create temporary variables to hold the values
            let item_id = Uuid::new_v4().to_string(); // id
            let unit_price = product_price.to_string(); // unit_price
            let total_price_str = total_price.to_string(); // total_price

            items_to_insert.push(NewSaleItem {
                id: item_id,
                sale_id: sale_id_str.clone(),
                product_id: product_to_sell.id.clone(),
                quantity: quantity_sold,
                unit_price: unit_price,
                total_price: total_price_str,
            });
            sale_total += &total_price;

            diesel::update(products::table)
                .filter(products::id.eq(&product_to_sell.id))
                .set(
                    products::stock_in_sale_units.eq(products::stock_in_sale_units - quantity_sold),
                )
                .execute(conn)?;
        }

        let sale_total_str = sale_total.to_string();
        let new_sale = NewSale {
            id: sale_id_str,
            user_id: admin_id.clone(),
            sale_number: sale_number_gen,
            total_amount: sale_total_str,
            date: date_str,
        };

        diesel::insert_into(sales::table)
            .values(&new_sale)
            .execute(conn)?;
        diesel::insert_into(sale_items::table)
            .values(&items_to_insert)
            .execute(conn)?;
    }

    log::info!("Ventes de test créées.");
    log::info!("--- Seeding terminé avec succès ---");

    Ok(())
}
