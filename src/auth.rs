use crate::error::{AppError, AppResult};
use crate::{db, models::User};
use bcrypt::verify;
use diesel::prelude::*;

// Authenticate with email and password
pub fn authenticate_by_email(email_input: &str, password_input: &str) -> AppResult<User> {
    use crate::schema::users::dsl::*;

    let mut conn = db::get_conn()?;

    // Try without as_select() first
    let user = users
        .filter(email.eq(email_input))
        .select(User::as_select())
        .first::<User>(&mut conn)
        .map_err(|e| {
            log::error!("Database error: : {:?}", e);
            AppError::Authentication("Utilisateur non trouvé".to_string())
        })?;

    // Verify the password
    if verify(password_input, &user.password).map_err(|_| "Erreur de vérification")? {
        Ok(user)
    } else {
        Err(AppError::Authentication("Mot de passe incorrect".into()))
    }
}

// Authenticate with name and password
pub fn authenticate_by_name(name_input: &str, password_input: &str) -> AppResult<User> {
    use crate::schema::users::dsl::*;

    let mut conn = db::get_conn()?;

    let user = users
        .filter(name.eq(name_input))
        .first::<User>(&mut conn)
        .map_err(|e| {
            log::error!("Database error: : {:?}", e);
            AppError::Authentication("Utilisateur non trouvé".to_string())
        })?;

    // Verify the password
    if verify(password_input, &user.password).map_err(|_| "Erreur de vérification")? {
        Ok(user)
    } else {
        Err(AppError::Authentication("Mot de passe incorrect".into()))
    }
}

// Generic authenticate function that tries both email and name
pub fn authenticate(login: &str, password_input: &str) -> AppResult<User> {
    // First try email
    if let Ok(user) = authenticate_by_email(login, password_input) {
        return Ok(user);
    }

    // If email fails, try name
    authenticate_by_name(login, password_input)
}

// Alternative approach using OR condition in a single query
pub fn authenticate_single_query(login: &str, password_input: &str) -> AppResult<User> {
    use crate::schema::users::dsl::*;

    let mut conn = db::get_conn()?;

    let user = users
        .filter(email.eq(login).or(name.eq(login)))
        .first::<User>(&mut conn)
        .map_err(|e| {
            log::error!("Database error: : {:?}", e);
            AppError::Authentication("Utilisateur non trouvé".to_string())
        })?;

    // Verify the password
    if verify(password_input, &user.password).map_err(|_| "Erreur de vérification")? {
        Ok(user)
    } else {
        Err(AppError::Authentication("Mot de passe incorrect".into()))
    }
}

// Alternative approach with better error handling:
pub fn authenticate_detailed(email_input: &str, password_input: &str) -> AppResult<User> {
    use crate::schema::users::dsl::*;

    let mut conn = db::get_conn()?;

    // Make sure to use the correct column name - if your email column is named differently
    let user = users
        .filter(email.eq(email_input)) // Assuming 'email' is your column name
        .select(User::as_select()) // Use this if you have #[derive(Selectable)]
        .first::<User>(&mut conn)
        .map_err(|e| {
            log::error!("Database error: : {:?}", e);
            AppError::Authentication("Utilisateur non trouvé".to_string())
        })?;

    // Verify the password
    match verify(password_input, &user.password) {
        Ok(true) => Ok(user),
        Ok(false) => Err(AppError::Authentication("Mot de passe incorrect".into())),
        Err(e) => {
            log::error!("Password verification error: {:?}", e);
            Err(AppError::Authentication("Erreur de vérification".into()))
        }
    }
}
