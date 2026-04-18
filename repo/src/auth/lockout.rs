use chrono::{Duration, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::errors::AppError;
use crate::models::user::User;
use crate::schema::users;

/// Returns an error if the account is currently locked.
pub fn check_lockout(user: &User) -> Result<(), AppError> {
    if let Some(locked_until) = user.locked_until {
        if Utc::now() < locked_until {
            return Err(AppError::AccountLocked(locked_until.to_rfc3339()));
        }
    }
    Ok(())
}

/// Increments failed attempt counter; locks account if threshold reached.
pub fn record_failed_attempt(
    conn: &mut PgConnection,
    user_id: Uuid,
    current_failures: i32,
    config: &AppConfig,
) -> Result<(), AppError> {
    let new_count = current_failures + 1;
    if new_count >= config.lockout_threshold {
        let lock_until = Utc::now()
            + Duration::seconds(config.lockout_duration_secs);
        diesel::update(users::table.find(user_id))
            .set((
                users::failed_attempts.eq(new_count),
                users::locked_until.eq(Some(lock_until)),
                users::updated_at.eq(Utc::now()),
            ))
            .execute(conn)?;
    } else {
        diesel::update(users::table.find(user_id))
            .set((
                users::failed_attempts.eq(new_count),
                users::updated_at.eq(Utc::now()),
            ))
            .execute(conn)?;
    }
    Ok(())
}

/// Resets the failed attempts counter and clears the lockout timestamp.
pub fn reset_failed_attempts(conn: &mut PgConnection, user_id: Uuid) -> Result<(), AppError> {
    diesel::update(users::table.find(user_id))
        .set((
            users::failed_attempts.eq(0),
            users::locked_until.eq(None::<chrono::DateTime<Utc>>),
            users::updated_at.eq(Utc::now()),
        ))
        .execute(conn)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn make_user(locked_until: Option<chrono::DateTime<Utc>>) -> crate::models::user::User {
        crate::models::user::User {
            id: Uuid::nil(),
            username: "testuser".into(),
            password_hash_enc: vec![],
            gov_id_enc: None,
            gov_id_last4: None,
            role_id: Uuid::nil(),
            department: None,
            location: None,
            is_active: true,
            failed_attempts: 0,
            locked_until,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_not_locked_when_no_lockout() {
        let user = make_user(None);
        assert!(check_lockout(&user).is_ok());
    }

    #[test]
    fn test_not_locked_when_time_passed() {
        let past = Utc::now() - Duration::seconds(60);
        let user = make_user(Some(past));
        assert!(check_lockout(&user).is_ok());
    }

    #[test]
    fn test_locked_when_future_time() {
        let future = Utc::now() + Duration::seconds(900);
        let user = make_user(Some(future));
        let result = check_lockout(&user);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::AccountLocked(_) => {}
            other => panic!("Expected AccountLocked, got {:?}", other),
        }
    }
}
