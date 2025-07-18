// src/main_window_manager/printer_callbacks.rs

use crate::{
    config::printer_config::{self, PrinterConfig, PrinterType},
    helpers::printer_error_handler::{PrinterError, PrinterErrorHandler},
    services::printing_service,
    ui::{self, MainWindow, PrinterUI},
};
use slint::{ComponentHandle, ModelRc, VecModel, Weak};

/// Convertit une configuration d'imprimante en modèle UI
fn config_to_ui(config: &PrinterConfig) -> PrinterUI {
    PrinterUI {
        name: config.name.clone().into(),
        port: config.port.clone().into(),
        printer_type: format!("{:?}", config.printer_type).into(),
        paper_width: config.paper_width.to_string().into(),
        is_default: config.is_default,
    }
}

/// Convertit un modèle UI en configuration d'imprimante
fn ui_to_config(ui_printer: &PrinterUI) -> PrinterConfig {
    PrinterConfig {
        name: ui_printer.name.to_string(),
        port: ui_printer.port.to_string(),
        printer_type: match ui_printer.printer_type.as_str() {
            "USB" => PrinterType::USB,
            "Network" => PrinterType::Network,
            "Windows" => PrinterType::Windows,
            _ => PrinterType::Serial,
        },
        paper_width: ui_printer.paper_width.parse().unwrap_or(48),
        is_default: ui_printer.is_default,
    }
}

/// Parse la largeur du papier depuis la sélection du ComboBox
fn parse_paper_width(width_selection: &str) -> u32 {
    if width_selection.starts_with("32") {
        32
    } else if width_selection.starts_with("48") {
        48
    } else if width_selection.starts_with("64") {
        64
    } else {
        48 // Valeur par défaut
    }
}

/// Affiche une erreur dans l'interface utilisateur
fn display_error(ui_handle: &Weak<MainWindow>, error: &PrinterError) {
    if let Some(ui) = ui_handle.upgrade() {
        log::error!("Erreur imprimante: {}", error.message());

        // Mettre à jour l'interface avec l'erreur
        ui.set_printer_error({
            ui::PrinterErrorUI {
                has_error: true,
                message: error.message().into(),
                icon: error.icon().into(),
                color: error.color().into(),
                error_type: match error {
                    PrinterError::ValidationError(_) => "Validation",
                    PrinterError::ConnectionError(_) => "Connexion",
                    PrinterError::ConfigurationError(_) => "Configuration",
                    PrinterError::NetworkError(_) => "Réseau",
                    PrinterError::SystemError(_) => "Système",
                    PrinterError::DuplicateError(_) => "Doublon",
                }
                .into(),
            }
        });
    }
}

/// Affiche un message de succès dans l'interface utilisateur
fn display_success(ui_handle: &Weak<MainWindow>, message: &str, icon: &str) {
    if let Some(ui) = ui_handle.upgrade() {
        log::info!("Succès: {}", message);

        ui.set_printer_success({
            ui::PrinterSuccessUI {
                has_success: true,
                message: message.into(),
                icon: icon.into(),
            }
        });
    }
}

/// Affiche le résultat d'un test d'imprimante
fn display_test_result(
    ui_handle: &Weak<MainWindow>,
    printer_name: &str,
    success: bool,
    message: &str,
) {
    if let Some(ui) = ui_handle.upgrade() {
        // Arrêter immédiatement l'indicateur de test
        ui.set_printer_test_result({
            ui::PrinterTestResultUI {
                is_testing: false,
                has_result: true,
                success,
                message: message.into(),
                printer_name: printer_name.into(),
            }
        });

        // Forcer la mise à jour de l'UI
        ui.window().request_redraw();

        log::info!(
            "Résultat du test pour '{}': {} - {}",
            printer_name,
            if success { "Succès" } else { "Échec" },
            message
        );
    }
}

/// Efface les notifications dans l'interface utilisateur
fn clear_notifications(ui_handle: &Weak<MainWindow>) {
    if let Some(ui) = ui_handle.upgrade() {
        ui.set_printer_error({
            ui::PrinterErrorUI {
                has_error: false,
                message: "".into(),
                icon: "".into(),
                color: "".into(),
                error_type: "".into(),
            }
        });

        ui.set_printer_success({
            ui::PrinterSuccessUI {
                has_success: false,
                message: "".into(),
                icon: "".into(),
            }
        });
    }
}

/// Configure tous les callbacks pour la gestion des imprimantes
pub fn setup(main_window_handle: &Weak<MainWindow>) {
    let ui = main_window_handle.upgrade().unwrap();

    setup_load_printers_callback(main_window_handle);
    setup_add_printer_callback(main_window_handle);
    setup_remove_printer_callback(main_window_handle);
    setup_set_default_printer_callback(main_window_handle);
    setup_test_printer_callback(main_window_handle);
    setup_clear_notifications_callback(main_window_handle);
    setup_port_suggestions_callback(main_window_handle);
    setup_help_message_callback(main_window_handle);

    // Charger les imprimantes au démarrage
    ui.invoke_load_printers();
}

/// Configure le callback pour charger les imprimantes
fn setup_load_printers_callback(main_window_handle: &Weak<MainWindow>) {
    let ui_handle = main_window_handle.clone();

    main_window_handle
        .upgrade()
        .unwrap()
        .on_load_printers(move || {
            if let Some(ui) = ui_handle.upgrade() {
                log::info!("Chargement des configurations d'imprimantes...");

                match std::panic::catch_unwind(|| printer_config::load_printers()) {
                    Ok(configs) => {
                        log::info!("Trouvé {} imprimantes configurées", configs.len());

                        let ui_model: Vec<PrinterUI> =
                            configs.into_iter().map(|c| config_to_ui(&c)).collect();

                        // Créer le modèle et l'assigner à l'interface
                        let model = ModelRc::new(VecModel::from(ui_model));
                        ui.set_printers_model(model);

                        log::info!("Modèle d'imprimantes chargé avec succès");
                    }
                    Err(_) => {
                        let error = PrinterError::SystemError(
                            "Impossible de charger les configurations d'imprimantes".to_string(),
                        );
                        display_error(&ui_handle, &error);
                    }
                }
            }
        });
}

/// Configure le callback pour ajouter une imprimante
fn setup_add_printer_callback(main_window_handle: &Weak<MainWindow>) {
    let add_handle = main_window_handle.clone();

    main_window_handle
        .upgrade()
        .unwrap()
        .on_add_printer(move |new_printer_ui| {
            if let Some(ui) = add_handle.upgrade() {
                log::info!(
                    "Tentative d'ajout d'une nouvelle imprimante: {}",
                    new_printer_ui.name
                );

                // Effacer les notifications précédentes
                clear_notifications(&add_handle);

                // Charger les configurations existantes pour validation
                let existing_configs =
                    match std::panic::catch_unwind(|| printer_config::load_printers()) {
                        Ok(configs) => configs,
                        Err(_) => {
                            let error = PrinterError::SystemError(
                                "Impossible de charger les configurations existantes".to_string(),
                            );
                            display_error(&add_handle, &error);
                            ui.set_is_adding_printer(false);
                            return;
                        }
                    };

                // Extraire les noms existants
                let existing_names: Vec<String> =
                    existing_configs.iter().map(|c| c.name.clone()).collect();

                // Valider les données d'entrée
                match PrinterErrorHandler::validate_printer_config(
                    &new_printer_ui.name,
                    &new_printer_ui.port,
                    &new_printer_ui.printer_type,
                    &new_printer_ui.paper_width,
                    &existing_names,
                ) {
                    Ok(_) => {
                        // Validation réussie, continuer avec l'ajout
                        add_printer_validated(&add_handle, new_printer_ui, existing_configs);
                    }
                    Err(error) => {
                        // Erreur de validation
                        display_error(&add_handle, &error);
                        ui.set_is_adding_printer(false);
                    }
                }
            }
        });
}

/// Ajoute une imprimante après validation réussie
fn add_printer_validated(
    ui_handle: &Weak<MainWindow>,
    new_printer_ui: PrinterUI,
    mut existing_configs: Vec<PrinterConfig>,
) {
    if let Some(ui) = ui_handle.upgrade() {
        // Créer la nouvelle configuration
        let mut new_config = ui_to_config(&new_printer_ui);

        // Si cette imprimante doit être par défaut, désactiver les autres
        if new_config.is_default {
            for config in &mut existing_configs {
                config.is_default = false;
            }
        } else if existing_configs.is_empty() {
            // Si c'est la première imprimante, la marquer comme par défaut
            new_config.is_default = true;
        }

        // Ajouter la nouvelle imprimante
        existing_configs.push(new_config);

        // Sauvegarder les configurations
        match printer_config::save_printers(&existing_configs) {
            Ok(_) => {
                log::info!(
                    "Nouvelle imprimante '{}' ajoutée avec succès",
                    new_printer_ui.name
                );

                display_success(
                    ui_handle,
                    &format!("Imprimante '{}' ajoutée avec succès", new_printer_ui.name),
                    "✅",
                );

                // Recharger la liste des imprimantes
                ui.invoke_load_printers();
            }
            Err(e) => {
                let error =
                    PrinterError::SystemError(format!("Erreur lors de la sauvegarde: {}", e));
                display_error(ui_handle, &error);
            }
        }

        ui.set_is_adding_printer(false);
    }
}

/// Configure le callback pour supprimer une imprimante
fn setup_remove_printer_callback(main_window_handle: &Weak<MainWindow>) {
    let remove_handle = main_window_handle.clone();

    main_window_handle
        .upgrade()
        .unwrap()
        .on_remove_printer(move |printer_name| {
            if let Some(ui) = remove_handle.upgrade() {
                log::info!("Tentative de suppression de l'imprimante: {}", printer_name);

                // Effacer les notifications précédentes
                clear_notifications(&remove_handle);

                let mut configs = match std::panic::catch_unwind(|| printer_config::load_printers())
                {
                    Ok(configs) => configs,
                    Err(_) => {
                        let error = PrinterError::SystemError(
                            "Impossible de charger les configurations".to_string(),
                        );
                        display_error(&remove_handle, &error);
                        return;
                    }
                };

                let initial_count = configs.len();

                // Supprimer l'imprimante
                configs.retain(|p| p.name != printer_name.as_str());

                if configs.len() == initial_count {
                    let error = PrinterError::ConfigurationError(format!(
                        "Imprimante '{}' introuvable",
                        printer_name
                    ));
                    display_error(&remove_handle, &error);
                    return;
                }

                // Si on supprime l'imprimante par défaut et qu'il en reste d'autres,
                // marquer la première comme par défaut
                if !configs.is_empty() && !configs.iter().any(|c| c.is_default) {
                    configs[0].is_default = true;
                    log::info!(
                        "Imprimante '{}' définie comme nouvelle imprimante par défaut",
                        configs[0].name
                    );
                }

                // Sauvegarder les configurations
                match printer_config::save_printers(&configs) {
                    Ok(_) => {
                        log::info!("Imprimante '{}' supprimée avec succès", printer_name);

                        display_success(
                            &remove_handle,
                            &format!("Imprimante '{}' supprimée avec succès", printer_name),
                            "🗑️",
                        );

                        // Recharger la liste des imprimantes
                        ui.invoke_load_printers();
                    }
                    Err(e) => {
                        let error = PrinterError::SystemError(format!(
                            "Erreur lors de la sauvegarde: {}",
                            e
                        ));
                        display_error(&remove_handle, &error);
                    }
                }
            }
        });
}

/// Configure le callback pour définir une imprimante par défaut
fn setup_set_default_printer_callback(main_window_handle: &Weak<MainWindow>) {
    let set_default_handle = main_window_handle.clone();

    main_window_handle
        .upgrade()
        .unwrap()
        .on_set_default_printer(move |printer_name| {
            if let Some(ui) = set_default_handle.upgrade() {
                log::info!(
                    "Définition de '{}' comme imprimante par défaut",
                    printer_name
                );

                // Effacer les notifications précédentes
                clear_notifications(&set_default_handle);

                let mut configs = match std::panic::catch_unwind(|| printer_config::load_printers())
                {
                    Ok(configs) => configs,
                    Err(_) => {
                        let error = PrinterError::SystemError(
                            "Impossible de charger les configurations".to_string(),
                        );
                        display_error(&set_default_handle, &error);
                        return;
                    }
                };

                let mut found = false;

                // Marquer toutes les imprimantes comme non par défaut,
                // sauf celle sélectionnée
                for config in &mut configs {
                    if config.name == printer_name.as_str() {
                        config.is_default = true;
                        found = true;
                    } else {
                        config.is_default = false;
                    }
                }

                if !found {
                    let error = PrinterError::ConfigurationError(format!(
                        "Imprimante '{}' introuvable",
                        printer_name
                    ));
                    display_error(&set_default_handle, &error);
                    return;
                }

                // Sauvegarder les configurations
                match printer_config::save_printers(&configs) {
                    Ok(_) => {
                        log::info!("Imprimante par défaut mise à jour avec succès");

                        display_success(
                            &set_default_handle,
                            &format!("'{}' définie comme imprimante par défaut", printer_name),
                            "⭐",
                        );

                        // Recharger la liste des imprimantes
                        ui.invoke_load_printers();
                    }
                    Err(e) => {
                        let error = PrinterError::SystemError(format!(
                            "Erreur lors de la sauvegarde: {}",
                            e
                        ));
                        display_error(&set_default_handle, &error);
                    }
                }
            }
        });
}

/// Configure le callback pour tester une imprimante
fn setup_test_printer_callback(main_window_handle: &Weak<MainWindow>) {
    let test_handle = main_window_handle.clone();

    main_window_handle
        .upgrade()
        .unwrap()
        .on_test_printer(move |printer_name| {
            if let Some(ui) = test_handle.upgrade() {
                log::info!("Test de l'imprimante: {}", printer_name);

                // Effacer les notifications précédentes
                clear_notifications(&test_handle);

                // Marquer le test comme en cours
                ui.set_printer_test_result({
                    ui::PrinterTestResultUI {
                        is_testing: true,
                        has_result: false,
                        success: false,
                        message: "".into(),
                        printer_name: printer_name.clone().into(),
                    }
                });

                //Forcer la mise à jour de l'UI immédiatement
                ui.window().request_redraw();

                let configs = match std::panic::catch_unwind(|| printer_config::load_printers()) {
                    Ok(configs) => configs,
                    Err(_) => {
                        let error = PrinterError::SystemError(
                            "Impossible de charger les configurations".to_string(),
                        );
                        display_error(&test_handle, &error);

                        //Arrêter explicitement le test et afficher le résultat
                        display_test_result(
                            &test_handle,
                            &printer_name,
                            false,
                            "Erreur système lors du chargement des configurations",
                        );
                        return;
                    }
                };

                if let Some(config_to_test) =
                    configs.iter().find(|p| p.name == printer_name.as_str())
                {
                    // Tester la connexion d'abord
                    match printing_service::test_printer_connection(config_to_test) {
                        Ok(true) => {
                            log::info!("Connexion à l'imprimante '{}' réussie", printer_name);

                            // Tenter d'imprimer une page de test
                            match printing_service::print_test_page(config_to_test) {
                                Ok(_) => {
                                    log::info!(
                                        "Page de test imprimée avec succès pour '{}'",
                                        printer_name
                                    );

                                    display_test_result(
                                        &test_handle,
                                        &printer_name,
                                        true,
                                        "Connexion et impression de test réussies",
                                    );
                                }
                                Err(e) => {
                                    log::error!(
                                        "Erreur lors de l'impression de test pour '{}': {}",
                                        printer_name,
                                        e
                                    );

                                    display_test_result(
                                        &test_handle,
                                        &printer_name,
                                        false,
                                        &format!("Connexion OK mais erreur d'impression: {}", e),
                                    );
                                }
                            }
                        }
                        Ok(false) => {
                            log::warn!(
                                "Impossible de se connecter à l'imprimante '{}'",
                                printer_name
                            );

                            display_test_result(
                                &test_handle,
                                &printer_name,
                                false,
                                "Impossible de se connecter à l'imprimante",
                            );
                        }
                        Err(e) => {
                            log::error!(
                                "Erreur lors du test de connexion pour '{}': {}",
                                printer_name,
                                e
                            );

                            display_test_result(
                                &test_handle,
                                &printer_name,
                                false,
                                &format!("Erreur de connexion: {}", e),
                            );
                        }
                    }
                } else {
                    let error = PrinterError::ConfigurationError(format!(
                        "Configuration introuvable pour l'imprimante '{}'",
                        printer_name
                    ));
                    display_error(&test_handle, &error);

                    // Arrêter explicitement le test et afficher le résultat
                    display_test_result(
                        &test_handle,
                        &printer_name,
                        false,
                        "Configuration d'imprimante introuvable",
                    );
                }
            }
        });
}
/// Configure le callback pour effacer les notifications
fn setup_clear_notifications_callback(main_window_handle: &Weak<MainWindow>) {
    let clear_handle = main_window_handle.clone();

    main_window_handle
        .upgrade()
        .unwrap()
        .on_clear_printer_notifications(move || {
            clear_notifications(&clear_handle);
        });
}

/// Configure le callback pour obtenir les suggestions de port
fn setup_port_suggestions_callback(main_window_handle: &Weak<MainWindow>) {
    let suggestions_handle = main_window_handle.clone();

    main_window_handle
        .upgrade()
        .unwrap()
        .on_get_printer_port_suggestions(move |printer_type| {
            if let Some(ui) = suggestions_handle.upgrade() {
                let suggestions = PrinterErrorHandler::get_port_suggestions(&printer_type);
                let slint_suggestions: Vec<slint::SharedString> =
                    suggestions.into_iter().map(|s| s.into()).collect();

                let model = ModelRc::new(VecModel::from(slint_suggestions));
                ui.set_printer_port_suggestions(model);
            }
        });
}

/// Configure le callback pour obtenir le message d'aide
fn setup_help_message_callback(main_window_handle: &Weak<MainWindow>) {
    let help_handle = main_window_handle.clone();

    main_window_handle
        .upgrade()
        .unwrap()
        .on_get_printer_help_message(move |printer_type| {
            if let Some(ui) = help_handle.upgrade() {
                let help_message = PrinterErrorHandler::get_help_message(&printer_type);
                ui.set_printer_help_message(help_message.into());
            }
        });
}

/// Fonction utilitaire pour obtenir l'imprimante par défaut
pub fn get_default_printer() -> Option<PrinterConfig> {
    let configs = printer_config::load_printers();
    configs.into_iter().find(|c| c.is_default)
}

/// Fonction utilitaire pour obtenir toutes les imprimantes
pub fn get_all_printers() -> Vec<PrinterConfig> {
    printer_config::load_printers()
}

/// Fonction utilitaire pour obtenir une imprimante par nom
pub fn get_printer_by_name(name: &str) -> Option<PrinterConfig> {
    let configs = printer_config::load_printers();
    configs.into_iter().find(|c| c.name == name)
}
