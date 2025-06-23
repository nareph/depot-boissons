use crate::{db, error::AppResult, schema::users::dsl::*};
use diesel::prelude::*;
use uuid::Uuid;

/// Met à jour le mot de passe d'un utilisateur et désactive le flag `must_change_password`.
pub fn update_user_password(user_id: Uuid, new_password_hash: &str) -> AppResult<()> {
    let mut conn = db::get_conn()?;

    diesel::update(users.find(user_id))
        .set((
            password.eq(new_password_hash),
            must_change_password.eq(false), // On désactive le flag ici !
        ))
        .execute(&mut conn)?;

    Ok(())
}
