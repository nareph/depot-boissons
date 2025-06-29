// src/main_window_manager/dashboard_callbacks.rs
use crate::{queries, ui};
use slint::Weak;

pub fn setup(main_window_handle: &Weak<ui::MainWindow>) {
    let dashboard_handle = main_window_handle.clone();
    main_window_handle
        .upgrade()
        .unwrap()
        .on_request_dashboard_data(move || {
            if let Some(ui) = dashboard_handle.upgrade() {
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
                                    p.total_stock_in_base_units, p.base_unit_name
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
        });
}
