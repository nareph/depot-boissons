use crate::error::{AppError, AppResult};
use crate::{db, models::User};
use bcrypt::verify;
use diesel::prelude::*;

// Authenticate with name and password
pub fn authenticate(name_input: &str, password_input: &str) -> AppResult<User> {
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
