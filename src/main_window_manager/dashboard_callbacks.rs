// src/main_window_manager/dashboard_callbacks.rs
use crate::{queries, ui};
use slint::Weak;

pub fn setup(main_window_handle: &Weak<ui::MainWindow>) {
    let dashboard_handle = main_window_handle.clone();

    // Fonction pour charger les données du dashboard
    let load_dashboard_data = {
        let handle = dashboard_handle.clone();
        move || {
            if let Some(ui) = handle.upgrade() {
                log::info!("Chargement des données du tableau de bord...");

                match queries::get_today_sales_summary() {
                    Ok((revenue, count)) => {
                        ui.set_today_revenue(format!("{} XAF", revenue).into());
                        ui.set_today_sales_count(count.to_string().into());
                    }
                    Err(e) => log::error!("Erreur chargement résumé ventes: {}", e),
                }

                match queries::get_low_stock_products(50) {
                    Ok(products) => {
                        let model = products
                            .into_iter()
                            .map(|p| ui::LowStockProductUI {
                                name: p.name.into(),
                                stock_info: format!(
                                    "{} {}",
                                    p.stock_in_sale_units, p.packaging_description
                                )
                                .into(),
                            })
                            .collect::<Vec<_>>();
                        ui.set_low_stock_products_model(
                            std::rc::Rc::new(slint::VecModel::from(model)).into(),
                        );
                    }
                    Err(e) => log::error!("Erreur chargement stock bas: {}", e),
                }
            }
        }
    };

    // Callback pour le chargement initial
    let initial_load = load_dashboard_data.clone();
    main_window_handle
        .upgrade()
        .unwrap()
        .on_request_dashboard_data(move || {
            initial_load();
        });

    // Callback pour le rafraîchissement manuel
    let refresh_handle = dashboard_handle.clone();
    main_window_handle
        .upgrade()
        .unwrap()
        .on_refresh_dashboard(move || {
            log::info!("Rafraîchissement manuel du tableau de bord...");
            if let Some(ui) = refresh_handle.upgrade() {
                // Optionnel : Afficher un indicateur de chargement
                ui.set_dashboard_loading(true);

                match queries::get_today_sales_summary() {
                    Ok((revenue, count)) => {
                        ui.set_today_revenue(format!("{} XAF", revenue).into());
                        ui.set_today_sales_count(count.to_string().into());
                    }
                    Err(e) => log::error!("Erreur rafraîchissement résumé ventes: {}", e),
                }

                match queries::get_low_stock_products(50) {
                    Ok(products) => {
                        let model = products
                            .into_iter()
                            .map(|p| ui::LowStockProductUI {
                                name: p.name.into(),
                                stock_info: format!(
                                    "{} {}",
                                    p.stock_in_sale_units, p.packaging_description
                                )
                                .into(),
                            })
                            .collect::<Vec<_>>();
                        ui.set_low_stock_products_model(
                            std::rc::Rc::new(slint::VecModel::from(model)).into(),
                        );
                    }
                    Err(e) => log::error!("Erreur rafraîchissement stock bas: {}", e),
                }

                ui.set_dashboard_loading(false);
                log::info!("Tableau de bord rafraîchi avec succès");
            }
        });
}

// Fonction utilitaire pour rafraîchir le dashboard depuis d'autres modules
pub fn refresh_dashboard(main_window_handle: &Weak<ui::MainWindow>) {
    if let Some(ui) = main_window_handle.upgrade() {
        ui.invoke_refresh_dashboard();
    }
}
