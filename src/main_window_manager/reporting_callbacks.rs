// src/main_window_manager/reporting_callbacks.rs

use crate::{queries, services::report_generator_service, ui};
use chrono::{Datelike, Duration, TimeZone, Utc};
use slint::{ComponentHandle, ModelRc, VecModel, Weak};
use std::sync::{Arc, Mutex};

use super::{show_error_dialog, show_info_dialog};

// État pour garder en mémoire les dernières données du rapport généré,
// afin de pouvoir les exporter sans refaire de requête.
#[derive(Debug, Clone, Default)]
struct ReportState {
    data: queries::ReportData,
}

/// Configure tous les callbacks pour la vue des rapports.
/// Cette fonction n'est appelée que si l'utilisateur est un administrateur.
pub fn setup(main_window_handle: &Weak<ui::MainWindow>) {
    // L'état est partagé entre tous les callbacks de ce module.
    let report_state = Arc::new(Mutex::new(ReportState::default()));
    let ui = main_window_handle.upgrade().unwrap();

    // --- Callback principal pour charger les données ---
    ui.on_request_report_data({
        let handle = main_window_handle.clone();
        let state = report_state.clone();
        move |period| {
            if let Some(ui) = handle.upgrade() {
                ui.set_report_is_loading(true);
                // Il est bon de forcer un redessinage pour que l'indicateur de chargement apparaisse immédiatement.
                ui.window().request_redraw();

                // Calculer la plage de dates en fonction de la période demandée
                let end_date = Utc::now();
                let start_date = match period.as_str() {
                    "7d" => end_date - Duration::days(7),
                    "30d" => end_date - Duration::days(30),
                    _ => {
                        // Par défaut, "year"
                        let year_start_date = Utc::now()
                            .date_naive()
                            .with_day(1)
                            .unwrap()
                            .with_month(1)
                            .unwrap();
                        // Convertir en NaiveDateTime avec minuit comme heure
                        let year_start_datetime = year_start_date.and_hms_opt(0, 0, 0).unwrap();

                        // Convertir en DateTime<Utc>
                        Utc.from_utc_datetime(&year_start_datetime)
                        // year_start.and_hms_opt(0, 0, 0).unwrap()
                    }
                };

                log::info!(
                    "Chargement du rapport pour la période: {} -> {}",
                    start_date,
                    end_date
                );

                // Appeler la requête pour obtenir les données
                match queries::get_report_data(start_date, end_date) {
                    Ok(data) => {
                        // Mettre à jour l'état partagé avec les nouvelles données
                        state.lock().unwrap().data = data.clone();

                        // Transformer les données pour l'UI
                        let kpis = vec![
                            ui::ReportKPI {
                                title: "Chiffre d'Affaires".into(),
                                value: format!("{} XAF", data.total_revenue).into(),
                                icon: "💰".into(),
                            },
                            ui::ReportKPI {
                                title: "Ventes Totales".into(),
                                value: data.total_sales.to_string().into(),
                                icon: "📈".into(),
                            },
                            ui::ReportKPI {
                                title: "Panier Moyen".into(),
                                value: if data.total_sales > 0 {
                                    format!(
                                        "{:.0} XAF",
                                        &data.total_revenue
                                            / bigdecimal::BigDecimal::from(data.total_sales)
                                    )
                                    .into()
                                } else {
                                    "0 XAF".into()
                                },
                                icon: "🛒".into(),
                            },
                        ];

                        let top_products = data
                            .top_products
                            .into_iter()
                            .enumerate()
                            .map(|(i, (prod, qty))| ui::TopProductUI {
                                rank: format!("#{}", i + 1).into(),
                                name: prod.name.into(),
                                quantity: format!("{} vendus", qty).into(),
                            })
                            .collect::<Vec<_>>();

                        // Mettre à jour les propriétés de l'UI
                        ui.set_report_kpis(ModelRc::new(VecModel::from(kpis)));
                        ui.set_report_top_products(ModelRc::new(VecModel::from(top_products)));
                    }
                    Err(e) => {
                        show_error_dialog("Erreur de Rapport", &e.to_string());
                    }
                }
                ui.set_report_is_loading(false);
            }
        }
    });

    // --- Callbacks d'exportation ---

    ui.on_export_pdf_clicked({
        let state = report_state.clone();
        move || {
            log::info!("Demande d'export PDF...");
            let data = &state.lock().unwrap().data;
            if data.total_sales == 0 {
                show_info_dialog(
                    "Export PDF",
                    "Aucune donnée à exporter pour la période sélectionnée.",
                );
                return;
            }
            match report_generator_service::generate_pdf_report(data) {
                Ok(path) => show_info_dialog(
                    "Export PDF Réussi",
                    &format!("Rapport sauvegardé : {}", path),
                ),
                Err(e) => show_error_dialog("Erreur d'Export PDF", &e.to_string()),
            }
        }
    });

    ui.on_export_excel_clicked({
        let state = report_state.clone();
        move || {
            log::info!("Demande d'export Excel...");
            let data = &state.lock().unwrap().data;
            if data.total_sales == 0 {
                show_info_dialog(
                    "Export Excel",
                    "Aucune donnée à exporter pour la période sélectionnée.",
                );
                return;
            }
            match report_generator_service::generate_excel_report(data) {
                Ok(path) => show_info_dialog(
                    "Export Excel Réussi",
                    &format!("Rapport sauvegardé : {}", path),
                ),
                Err(e) => show_error_dialog("Erreur d'Export Excel", &e.to_string()),
            }
        }
    });

    // Déclenche le chargement initial avec la période par défaut ("7 derniers jours")
    ui.invoke_request_report_data("7d".into());
}
