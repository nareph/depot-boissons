// src/queries/password_queries.rs
use crate::{db, error::AppResult};
use diesel::prelude::*;
use uuid::Uuid;

/// Met à jour le mot de passe d'un utilisateur et désactive le flag `must_change_password`.
pub fn update_user_password(user_id: Uuid, new_password_hash: &str) -> AppResult<()> {
    use crate::schema::users::dsl::*;
    let mut conn = db::get_conn()?;

    diesel::update(users.find(user_id))
        .set((
            password.eq(new_password_hash),
            must_change_password.eq(false),
        ))
        .execute(&mut conn)?;

    Ok(())
}
