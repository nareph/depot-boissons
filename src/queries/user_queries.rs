// src/queries/user_queries.rs
use super::SortOrder;
use crate::{
    db,
    error::AppResult,
    models::{NewUser, User},
};
use diesel::prelude::*;
use rand::Rng;
use rand::distr::Alphanumeric;
use uuid::Uuid;

// --- STRUCTURES POUR LA PAGINATION ET LE FILTRAGE ---

#[derive(Debug, Clone)]
pub struct UserFilter {
    pub search_term: Option<String>,
    pub role_filter: Option<String>,
    pub sort_by: UserSortBy,
    pub sort_order: SortOrder,
}

#[derive(Debug, Clone)]
pub enum UserSortBy {
    Name,
    Role,
    CreatedAt,
}

// #[derive(Debug, Clone)]
// pub enum SortOrder {
//     Asc,
//     Desc,
// }

#[derive(Debug, Clone)]
pub struct UserPagination {
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Clone)]
pub struct UserListResult {
    pub users: Vec<User>,
    pub total_count: i64,
    pub current_page: i64,
    pub total_pages: i64,
    pub per_page: i64,
}

impl Default for UserFilter {
    fn default() -> Self {
        Self {
            search_term: None,
            role_filter: None,
            sort_by: UserSortBy::Name,
            sort_order: SortOrder::Asc,
        }
    }
}

impl Default for UserPagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 5,
        }
    }
}

// --- CRUD & GESTION AMÉLIORÉE ---

/// Récupère tous les utilisateurs avec pagination, filtrage et tri
pub fn get_users_paginated(
    except_user_id: Uuid,
    filter: UserFilter,
    pagination: UserPagination,
) -> AppResult<UserListResult> {
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;

    // Fonction helper pour construire la requête avec filtres
    let build_filtered_query = || {
        let mut query = users.filter(id.ne(except_user_id)).into_boxed();

        // Application des filtres
        if let Some(search) = &filter.search_term {
            if !search.trim().is_empty() {
                let search_pattern = format!("%{}%", search.trim());
                query = query.filter(name.ilike(search_pattern));
            }
        }

        if let Some(role_filter) = &filter.role_filter {
            if !role_filter.trim().is_empty() && role_filter != "all" {
                query = query.filter(role.eq(role_filter));
            }
        }

        query
    };

    // Comptage total (avec les filtres appliqués)
    let total_count = build_filtered_query()
        .count()
        .get_result::<i64>(&mut conn)?;

    // Requête pour les données avec tri et pagination
    let query_with_sort = match (&filter.sort_by, &filter.sort_order) {
        (UserSortBy::Name, SortOrder::Asc) => build_filtered_query().order(name.asc()),
        (UserSortBy::Name, SortOrder::Desc) => build_filtered_query().order(name.desc()),
        (UserSortBy::Role, SortOrder::Asc) => build_filtered_query().order(role.asc()),
        (UserSortBy::Role, SortOrder::Desc) => build_filtered_query().order(role.desc()),
        (UserSortBy::CreatedAt, SortOrder::Asc) => build_filtered_query().order(id.asc()),
        (UserSortBy::CreatedAt, SortOrder::Desc) => build_filtered_query().order(id.desc()),
    };

    // Application de la pagination
    let offset = (pagination.page - 1) * pagination.per_page;
    let users_result = query_with_sort
        .limit(pagination.per_page)
        .offset(offset)
        .select(User::as_select())
        .load::<User>(&mut conn)?;

    // Calcul du nombre de pages
    let total_pages = (total_count + pagination.per_page - 1) / pagination.per_page;

    Ok(UserListResult {
        users: users_result,
        total_count,
        current_page: pagination.page,
        total_pages,
        per_page: pagination.per_page,
    })
}

/// Version simple pour la rétrocompatibilité
pub fn get_all_users(except_user_id: Uuid) -> AppResult<Vec<User>> {
    let result = get_users_paginated(
        except_user_id,
        UserFilter::default(),
        UserPagination {
            page: 1,
            per_page: 1000, // Limite élevée pour récupérer tous les utilisateurs
        },
    )?;
    Ok(result.users)
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

// --- FONCTIONS UTILITAIRES ---

/// Récupère la liste des rôles disponibles (pour les filtres)
pub fn get_available_roles() -> Vec<String> {
    vec![
        "admin".to_string(),
        "user".to_string(),
        "moderator".to_string(), // Ajoutez d'autres rôles selon vos besoins
    ]
}

/// Compte le nombre total d'utilisateurs (utile pour les statistiques)
pub fn count_users(except_user_id: Uuid) -> AppResult<i64> {
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;
    let count = users
        .filter(id.ne(except_user_id))
        .count()
        .get_result::<i64>(&mut conn)?;
    Ok(count)
}
