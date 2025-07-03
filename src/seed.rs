// src/seed.rs

use crate::{
    error::AppResult,
    // On importe seulement les modèles dont on a besoin maintenant
    models::{NewProduct, NewSale, NewSaleItem, NewUser, Product, Sale},
};
use bcrypt::{DEFAULT_COST, hash};
use bigdecimal::BigDecimal;
use chrono::Utc;
use diesel::prelude::*;
use rand::Rng;
use std::str::FromStr;
use uuid::Uuid;

pub fn seed_database(conn: &mut PgConnection) -> AppResult<()> {
    // Le schéma importé est maintenant plus simple
    use crate::schema::{products, sale_items, sales, users};

    log::info!("--- Début du seeding de la base de données ---");

    // 1. Nettoyer les tables dans le bon ordre
    log::info!("Nettoyage des tables existantes...");
    diesel::delete(sale_items::table).execute(conn)?;
    diesel::delete(sales::table).execute(conn)?;
    diesel::delete(products::table).execute(conn)?;
    diesel::delete(users::table).execute(conn)?;
    log::info!("Tables nettoyées.");

    // 2. Créer l'utilisateur administrateur (ne change pas)
    log::info!("Création de l'utilisateur admin...");
    let admin_password = hash("admin123", DEFAULT_COST)?;
    let admin_user = NewUser {
        id: Uuid::new_v4(),
        password: &admin_password,
        name: "Administrateur",
        role: "Admin",
        must_change_password: true,
    };
    diesel::insert_into(users::table)
        .values(&admin_user)
        .execute(conn)?;
    log::info!("Utilisateur admin créé.");

    // 3. Créer les produits finis (SKUs)
    log::info!("Création des produits (SKUs)...");
    let products_data = vec![
        NewProduct {
            id: Uuid::new_v4(),
            name: "Castel Beer".to_string(),
            packaging_description: "Casier 65cl de 12".to_string(),
            sku: Some("CAS-65-CAS12".to_string()),
            stock_in_sale_units: 100,
            price_per_sale_unit: BigDecimal::from_str("8500.00")?,
        },
        NewProduct {
            id: Uuid::new_v4(),
            name: "33 Export".to_string(),
            packaging_description: "Casier 65cl de 12".to_string(),
            sku: Some("33EXP-65-CAS12".to_string()),
            stock_in_sale_units: 150,
            price_per_sale_unit: BigDecimal::from_str("8500.00")?,
        },
        NewProduct {
            id: Uuid::new_v4(),
            name: "Guinness Smooth".to_string(),
            packaging_description: "Casier 33cl de 24".to_string(),
            sku: Some("GUIN-33-CAS24".to_string()),
            stock_in_sale_units: 80,
            price_per_sale_unit: BigDecimal::from_str("14500.00")?,
        },
        NewProduct {
            id: Uuid::new_v4(),
            name: "Supermont Eau".to_string(),
            packaging_description: "Palette 1.5L de 12".to_string(),
            sku: Some("SPMT-1.5-PAL12".to_string()),
            stock_in_sale_units: 50,
            price_per_sale_unit: BigDecimal::from_str("2500.00")?,
        },
        // On ajoute un produit avec un stock faible pour tester le dashboard
        NewProduct {
            id: Uuid::new_v4(),
            name: "Coca-Cola".to_string(),
            packaging_description: "Casier 33cl de 24".to_string(),
            sku: Some("COKE-33-CAS24".to_string()),
            stock_in_sale_units: 45, // <-- Stock faible
            price_per_sale_unit: BigDecimal::from_str("11000.00")?,
        },
    ];
    let inserted_products = diesel::insert_into(products::table)
        .values(&products_data)
        .get_results::<Product>(conn)?;
    log::info!("Produits créés.");

    // 4. Créer des ventes de test pour aujourd'hui
    log::info!("Création des ventes de test pour aujourd'hui...");
    let mut rng = rand::rng();

    for i in 1..=5 {
        let new_sale = NewSale {
            id: Uuid::new_v4(),
            sale_number: &format!("VTE-{:05}", i),
            total_amount: BigDecimal::from(0),
            date: Utc::now(), // La vente a lieu "maintenant"
        };
        let sale = diesel::insert_into(sales::table)
            .values(&new_sale)
            .get_result::<Sale>(conn)?;

        let num_items = rng.random_range(1..=3);
        let mut sale_total = BigDecimal::from(0);

        for _ in 0..num_items {
            // On choisit un produit au hasard dans la liste des produits insérés
            let product_to_sell = &inserted_products[rng.random_range(0..inserted_products.len())];
            // On vend entre 1 et 5 unités (casiers/palettes)
            let quantity_sold = rng.random_range(1..=5);

            let total_price =
                &product_to_sell.price_per_sale_unit * BigDecimal::from(quantity_sold);

            let new_sale_item = NewSaleItem {
                id: Uuid::new_v4(),
                sale_id: sale.id,
                product_id: product_to_sell.id, // On utilise directement product_id
                quantity: quantity_sold,
                unit_price: product_to_sell.price_per_sale_unit.clone(),
                total_price: total_price.clone(),
            };
            diesel::insert_into(sale_items::table)
                .values(&new_sale_item)
                .execute(conn)?;

            sale_total += &total_price;

            // Mettre à jour le stock du produit
            diesel::update(products::table.find(product_to_sell.id))
                .set(
                    products::stock_in_sale_units.eq(products::stock_in_sale_units - quantity_sold),
                )
                .execute(conn)?;
        }

        // Mettre à jour le montant total de la vente
        diesel::update(sales::table.find(sale.id))
            .set(sales::total_amount.eq(sale_total))
            .execute(conn)?;
    }
    log::info!("Ventes de test créées.");
    log::info!("--- Seeding terminé avec succès ---");

    Ok(())
}
