// src/seed.rs

use crate::{
    error::AppResult,
    models::{
        NewPackagingUnit, NewProduct, NewProductOffering, NewSale, NewSaleItem, NewUser,
        PackagingUnit, Product, ProductOffering, Sale,
    },
};
use bcrypt::{DEFAULT_COST, hash};
use bigdecimal::BigDecimal;
use chrono::Utc;
use diesel::prelude::*;
use rand::Rng;
use std::str::FromStr;
use uuid::Uuid;

pub fn seed_database(conn: &mut PgConnection) -> AppResult<()> {
    use crate::schema::{packaging_units, product_offerings, products, sale_items, sales, users};

    log::info!("--- Début du seeding de la base de données ---");

    // 1. Nettoyer les tables dans le bon ordre pour respecter les clés étrangères
    log::info!("Nettoyage des tables existantes...");
    diesel::delete(sale_items::table).execute(conn)?;
    diesel::delete(sales::table).execute(conn)?;
    diesel::delete(product_offerings::table).execute(conn)?;
    diesel::delete(packaging_units::table).execute(conn)?;
    diesel::delete(products::table).execute(conn)?;
    diesel::delete(users::table).execute(conn)?;
    log::info!("Tables nettoyées.");

    // 2. Créer l'utilisateur administrateur
    log::info!("Création de l'utilisateur admin...");
    let admin_password = hash("admin123", DEFAULT_COST)?;
    let admin_user = NewUser {
        id: Uuid::new_v4(),
        email: "admin@depot-boissons.com",
        password: &admin_password,
        name: "Administrateur",
        role: "Admin",
        must_change_password: true,
    };
    diesel::insert_into(users::table)
        .values(&admin_user)
        .execute(conn)?;
    log::info!("Utilisateur admin créé.");

    // 3. Créer les unités de conditionnement
    log::info!("Création des unités de conditionnement...");
    let packaging_data = vec![
        NewPackagingUnit {
            id: Uuid::new_v4(),
            name: "Bouteille 65cl",
            contained_base_units: 1,
        },
        NewPackagingUnit {
            id: Uuid::new_v4(),
            name: "Demi-casier (6 Bouteilles)",
            contained_base_units: 6,
        },
        NewPackagingUnit {
            id: Uuid::new_v4(),
            name: "Casier (12 Bouteilles)",
            contained_base_units: 12,
        },
        NewPackagingUnit {
            id: Uuid::new_v4(),
            name: "Bouteille 33cl",
            contained_base_units: 1,
        },
        NewPackagingUnit {
            id: Uuid::new_v4(),
            name: "Casier (24 Bouteilles)",
            contained_base_units: 24,
        },
    ];
    let inserted_packaging_units = diesel::insert_into(packaging_units::table)
        .values(&packaging_data)
        .get_results::<PackagingUnit>(conn)?;
    log::info!("Unités de conditionnement créées.");

    // 4. Créer les produits de base
    log::info!("Création des produits de base...");
    let products_data = vec![
        NewProduct {
            id: Uuid::new_v4(),
            name: "Beaufort",
            base_unit_name: "bouteille 65cl",
            total_stock_in_base_units: 240,
        },
        NewProduct {
            id: Uuid::new_v4(),
            name: "Guinness",
            base_unit_name: "bouteille 33cl",
            total_stock_in_base_units: 480,
        },
        NewProduct {
            id: Uuid::new_v4(),
            name: "Coca-Cola",
            base_unit_name: "bouteille 33cl",
            total_stock_in_base_units: 120,
        },
        // On ajoute un produit avec un stock faible pour tester le dashboard
        NewProduct {
            id: Uuid::new_v4(),
            name: "Fanta",
            base_unit_name: "bouteille 33cl",
            total_stock_in_base_units: 45,
        },
    ];
    let inserted_products = diesel::insert_into(products::table)
        .values(&products_data)
        .get_results::<Product>(conn)?;
    log::info!("Produits créés.");

    // 5. Créer les offres de produits
    log::info!("Création des offres de produits...");
    let beaufort = inserted_products
        .iter()
        .find(|p| p.name == "Beaufort")
        .unwrap();
    let guinness = inserted_products
        .iter()
        .find(|p| p.name == "Guinness")
        .unwrap();
    let coca = inserted_products
        .iter()
        .find(|p| p.name == "Coca-Cola")
        .unwrap();
    let fanta = inserted_products
        .iter()
        .find(|p| p.name == "Fanta")
        .unwrap();

    let bouteille_65cl = inserted_packaging_units
        .iter()
        .find(|u| u.name == "Bouteille 65cl")
        .unwrap();
    let casier_12 = inserted_packaging_units
        .iter()
        .find(|u| u.name == "Casier (12 Bouteilles)")
        .unwrap();
    let bouteille_33cl = inserted_packaging_units
        .iter()
        .find(|u| u.name == "Bouteille 33cl")
        .unwrap();
    let casier_24 = inserted_packaging_units
        .iter()
        .find(|u| u.name == "Casier (24 Bouteilles)")
        .unwrap();

    let offerings_data = vec![
        NewProductOffering {
            id: Uuid::new_v4(),
            product_id: beaufort.id,
            packaging_unit_id: bouteille_65cl.id,
            price: BigDecimal::from_str("700.00")?,
        },
        NewProductOffering {
            id: Uuid::new_v4(),
            product_id: beaufort.id,
            packaging_unit_id: casier_12.id,
            price: BigDecimal::from_str("8000.00")?,
        },
        NewProductOffering {
            id: Uuid::new_v4(),
            product_id: guinness.id,
            packaging_unit_id: bouteille_33cl.id,
            price: BigDecimal::from_str("650.00")?,
        },
        NewProductOffering {
            id: Uuid::new_v4(),
            product_id: guinness.id,
            packaging_unit_id: casier_24.id,
            price: BigDecimal::from_str("15000.00")?,
        },
        NewProductOffering {
            id: Uuid::new_v4(),
            product_id: coca.id,
            packaging_unit_id: bouteille_33cl.id,
            price: BigDecimal::from_str("500.00")?,
        },
        NewProductOffering {
            id: Uuid::new_v4(),
            product_id: fanta.id,
            packaging_unit_id: bouteille_33cl.id,
            price: BigDecimal::from_str("500.00")?,
        },
    ];
    let inserted_offerings = diesel::insert_into(product_offerings::table)
        .values(&offerings_data)
        .get_results::<ProductOffering>(conn)?;
    log::info!("Offres créées.");

    // 6. Créer des ventes de test pour aujourd'hui
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
            let offering = &inserted_offerings[rng.random_range(0..inserted_offerings.len())];
            let quantity = rng.random_range(1..=4);

            let new_sale_item = NewSaleItem {
                id: Uuid::new_v4(),
                sale_id: sale.id,
                product_offering_id: offering.id,
                quantity,
                unit_price: offering.price.clone(),
                total_price: &offering.price * BigDecimal::from(quantity),
            };
            diesel::insert_into(sale_items::table)
                .values(&new_sale_item)
                .execute(conn)?;

            sale_total += &new_sale_item.total_price;

            let packaging_unit = inserted_packaging_units
                .iter()
                .find(|u| u.id == offering.packaging_unit_id)
                .unwrap();
            let stock_to_remove = quantity * packaging_unit.contained_base_units;

            diesel::update(products::table.find(offering.product_id))
                .set(
                    products::total_stock_in_base_units
                        .eq(products::total_stock_in_base_units - stock_to_remove),
                )
                .execute(conn)?;
        }

        diesel::update(sales::table.find(sale.id))
            .set(sales::total_amount.eq(sale_total))
            .execute(conn)?;
    }
    log::info!("Ventes de test créées.");
    log::info!("--- Seeding terminé avec succès ---");

    Ok(())
}
