// src/main_window_manager/user_callbacks.rs
use crate::{queries, ui};
use slint::{ComponentHandle, Weak};
use uuid::Uuid;

pub fn setup(main_window_handle: &Weak<ui::MainWindow>, current_user_id: Uuid) {
    let users_handle = main_window_handle.clone();
    main_window_handle
        .upgrade()
        .unwrap()
        .on_request_users(move || {
            if let Some(ui) = users_handle.upgrade() {
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
    main_window_handle
        .upgrade()
        .unwrap()
        .on_delete_user_clicked(move |user_id_str| {
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
    main_window_handle
        .upgrade()
        .unwrap()
        .on_add_user_clicked(move || {
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
