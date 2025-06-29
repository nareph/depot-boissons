// src/queries/user_queries.rs
use crate::{
    db,
    error::AppResult,
    models::{NewUser, User},
};
use diesel::prelude::*;
use uuid::Uuid;

/// Récupère tous les utilisateurs sauf celui qui est spécifié (pour ne pas que l'admin se supprime lui-même).
pub fn get_all_users(except_user_id: Uuid) -> AppResult<Vec<User>> {
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;
    let all_users = users
        .filter(id.ne(except_user_id))
        .order(name.asc())
        .load::<User>(&mut conn)?;
    Ok(all_users)
}

/// Crée un nouvel utilisateur. Le mot de passe doit déjà être haché.
pub fn create_user(
    new_name: &str,
    new_email: &str,
    hashed_password: &str,
    new_role: &str,
) -> AppResult<User> {
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;

    let new_user = NewUser {
        id: Uuid::new_v4(),
        name: new_name,
        email: new_email,
        password: hashed_password,
        role: new_role,
        must_change_password: true,
    };

    let created_user = diesel::insert_into(users)
        .values(&new_user)
        .get_result(&mut conn)?;

    Ok(created_user)
}

/// Supprime un utilisateur par son ID.
pub fn delete_user(user_id_to_delete: Uuid) -> AppResult<usize> {
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;
    let num_deleted = diesel::delete(users.find(user_id_to_delete)).execute(&mut conn)?;
    Ok(num_deleted)
}

/// Récupère un utilisateur par son ID.
pub fn get_user_by_id(user_id: Uuid) -> AppResult<User> {
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;
    let user = users.find(user_id).first::<User>(&mut conn)?;
    Ok(user)
}
