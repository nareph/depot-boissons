// src/queries/user_queries.rs

use crate::{
    db,
    error::AppResult,
    models::{NewUser, User},
};
use diesel::prelude::*;
use rand::Rng;
use rand::distr::Alphanumeric;
use uuid::Uuid;

// --- CRUD & GESTION ---

/// Récupère tous les utilisateurs sauf celui qui est spécifié (pour ne pas que l'admin se supprime lui-même).
pub fn get_all_users(except_user_id: Uuid) -> AppResult<Vec<User>> {
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;
    let all_users = users
        .filter(id.ne(except_user_id))
        .order(name.asc())
        .select(User::as_select()) 
        .load::<User>(&mut conn)?;
    Ok(all_users)
}

/// Récupère un utilisateur par son ID.
pub fn get_user_by_id(user_id: Uuid) -> AppResult<User> {
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;
    let user = users
        .find(user_id)
        .select(User::as_select())
        .first::<User>(&mut conn)?;
    Ok(user)
}

/// Crée un nouvel utilisateur. Le mot de passe doit déjà être haché.
pub fn create_user(new_name: &str, hashed_password: &str, new_role: &str) -> AppResult<User> {
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;

    let new_user = NewUser {
        id: Uuid::new_v4(),
        name: new_name,
        password: hashed_password,
        role: new_role,
        must_change_password: true, // Toujours forcer le changement pour les nouveaux comptes
    };

    diesel::insert_into(users)
        .values(&new_user)
        .get_result(&mut conn)
        .map_err(Into::into)
}

/// Met à jour le nom et le rôle d'un utilisateur.
pub fn update_user_info(user_id: Uuid, new_name: &str, new_role: &str) -> AppResult<User> {
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;

    diesel::update(users.find(user_id))
        .set((name.eq(new_name), role.eq(new_role)))
        .get_result(&mut conn)
        .map_err(Into::into)
}

/// Supprime un utilisateur par son ID.
pub fn delete_user(user_id_to_delete: Uuid) -> AppResult<usize> {
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;
    diesel::delete(users.find(user_id_to_delete))
        .execute(&mut conn)
        .map_err(Into::into)
}

// --- GESTION DE MOT DE PASSE ---

/// Réinitialise le mot de passe d'un utilisateur et retourne le mot de passe temporaire.
pub fn reset_user_password(user_id: Uuid) -> AppResult<String> {
    // 1. Générer un mot de passe temporaire simple

    let temp_password: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(12) // 12 caractères alphanumériques
        .map(char::from)
        .collect();

    // 2. Hacher ce mot de passe
    let hashed_password = bcrypt::hash(&temp_password, bcrypt::DEFAULT_COST)?;

    // 3. Mettre à jour la BDD
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;

    diesel::update(users.find(user_id))
        .set((
            password.eq(hashed_password),
            must_change_password.eq(true), // Très important
        ))
        .execute(&mut conn)?;

    // 4. Retourner le mot de passe temporaire en clair pour l'afficher à l'admin
    Ok(temp_password)
}
