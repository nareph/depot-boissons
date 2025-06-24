// src/main.rs

// Déclarer tous les modules de l'application
pub mod auth;
pub mod change_password;
pub mod change_password_user;
pub mod db;
pub mod error;
pub mod login;
pub mod models;
pub mod queries;
pub mod schema;
pub mod seed;

use error::AppResult;
use models::User;
use slint::ComponentHandle;
use uuid::Uuid;

// Le module `ui` qui va contenir tous les composants Slint
pub mod ui {
    slint::include_modules!();
}

/// Point d'entrée principal de l'application.
fn main() -> AppResult<()> {
    // Initialise le logger pour afficher les messages dans la console.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Application en cours de démarrage...");

    // Initialise la connexion à la base de données et exécute les migrations.
    db::init()?;

    // Vérifie si l'argument `--seed` est passé pour peupler la base de données.
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--seed".to_string()) {
        log::info!("Argument --seed détecté. Lancement du seeding...");
        let mut conn = db::get_conn()?;
        if let Err(e) = seed::seed_database(&mut conn) {
            log::error!("Erreur lors du seeding de la base de données : {}", e);
        }
    }

    // Lance le flux de l'application en commençant par l'écran de connexion.
    show_login_and_start_app()?;

    log::info!("Application terminée.");
    Ok(())
}

/// Affiche la fenêtre de connexion et, en cas de succès, lance le tableau de bord.
fn show_login_and_start_app() -> AppResult<()> {
    match login::show()? {
        Some(user) => {
            if user.must_change_password {
                if !change_password::show_and_update(&user)? {
                    return Ok(());
                }
            }
            run_main_window(user)?;
        }
        None => {
            log::info!("Fenêtre de connexion fermée. L'application se termine.");
        }
    }
    Ok(())
}

/// Gère la fenêtre principale et sa logique.
fn run_main_window(user: User) -> AppResult<()> {
    log::info!(
        "Lancement du tableau de bord pour l'utilisateur '{}'.",
        user.name
    );
    let main_window = ui::MainWindow::new()?;

    let is_admin = user.role == "Admin";
    main_window.set_is_admin(is_admin);
    main_window.set_welcome_message(format!("Utilisateur: {}", user.name).into());

    let main_window_handle = main_window.as_weak();
    let current_user_id = user.id;

    // --- Définition des callbacks ---
    let logout_handle = main_window_handle.clone();
    main_window.on_logout_clicked(move || {
        if let Some(ui) = logout_handle.upgrade() {
            log::info!("Déconnexion demandée.");
            let _ = ui.hide();
        }
    });

    main_window.on_change_password_clicked(move || {
        if let Err(e) = change_password_user::show(current_user_id) {
            log::error!(
                "Erreur lors de l'ouverture de la fenêtre de changement de mot de passe: {}",
                e
            );
        }
    });

    let dashboard_handle = main_window_handle.clone();
    main_window.on_request_dashboard_data(move || {
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

    let products_handle = main_window_handle.clone();
    main_window.on_request_products(move || {
        if let Some(ui) = products_handle.upgrade() {
            log::info!("Chargement des produits...");
            match queries::get_all_products_with_offers() {
                Ok(data) => {
                    let model = data
                        .into_iter()
                        .map(|(p, offers)| ui::ProductUI {
                            id: p.id.to_string().into(),
                            name: p.name.into(),
                            stock: format!("{} {}", p.total_stock_in_base_units, p.base_unit_name)
                                .into(),
                            price_offers: offers
                                .into_iter()
                                .map(|(o, u)| format!("{} XAF / {}", o.price, u.name))
                                .collect::<Vec<_>>()
                                .join("\n")
                                .into(),
                        })
                        .collect::<Vec<_>>();
                    ui.set_products_model(std::rc::Rc::new(slint::VecModel::from(model)).into());
                }
                Err(e) => log::error!("Erreur chargement produits: {}", e),
            }
        }
    });

    if is_admin {
        let users_handle = main_window_handle.clone();
        main_window.on_request_users(move || {
            if let Some(ui) = users_handle.upgrade() {
                log::info!("Chargement des utilisateurs...");
                match queries::get_all_users(current_user_id) {
                    Ok(data) => {
                        let model = data
                            .into_iter()
                            .map(|u| ui::UserUI {
                                id: u.id.to_string().into(),
                                name: u.name.into(),
                                email: u.email.into(),
                                role: u.role.into(),
                            })
                            .collect::<Vec<_>>();
                        ui.set_users_model(std::rc::Rc::new(slint::VecModel::from(model)).into());
                    }
                    Err(e) => log::error!("Erreur chargement utilisateurs: {}", e),
                }
            }
        });

        let delete_handle = main_window_handle.clone();
        main_window.on_delete_user_clicked(move |user_id_str| {
            if let Some(ui) = delete_handle.upgrade() {
                if let Ok(user_id) = Uuid::parse_str(&user_id_str) {
                    if let Ok(count) = queries::delete_user(user_id) {
                        if count > 0 {
                            ui.invoke_request_users();
                        }
                    }
                }
            }
        });

        let add_handle = main_window_handle.clone();
        main_window.on_add_user_clicked(move || {
            if let Some(main_ui) = add_handle.upgrade() {
                let add_window = ui::AddUserWindow::new().unwrap();
                let main_ui_handle_clone = main_ui.as_weak();

                let add_window_handle = add_window.as_weak();
                add_window.on_save_clicked(move |name, email, password, role| {
                    if let Some(add_win) = add_window_handle.upgrade() {
                        if let Ok(hash) = bcrypt::hash(&password, bcrypt::DEFAULT_COST) {
                            if queries::create_user(&name, &email, &hash, &role).is_ok() {
                                main_ui_handle_clone
                                    .upgrade()
                                    .unwrap()
                                    .invoke_request_users();
                                let _ = add_win.hide();
                            } else {
                                add_win.set_status_text("Erreur: l'email existe déjà?".into());
                            }
                        } else {
                            add_win.set_status_text("Erreur interne.".into());
                        }
                    }
                });
                let _ = add_window.run();
            }
        });
    }

    // Lance la boucle d'événements de la fenêtre principale.
    // L'exécution est bloquée ici jusqu'à la fermeture de la fenêtre.
    main_window.run()?;

    // Une fois la fenêtre fermée (par déconnexion ou bouton "X"), on relance le login.
    log::info!("Fenêtre principale fermée, retour à l'écran de connexion.");
    show_login_and_start_app()?;

    Ok(())
}
