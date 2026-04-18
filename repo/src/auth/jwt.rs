use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::errors::AppError;
use crate::models::delegation::Delegation;
use crate::models::role::Role;
use crate::models::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub role_id: Uuid,
    pub role_name: String,
    pub data_scope: String,
    pub scope_value: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub delegated_permissions: Vec<Uuid>,
    pub exp: i64,
    pub iat: i64,
    pub token_type: String,
}

pub fn issue_access_token(
    user: &User,
    role: &Role,
    delegations: &[Delegation],
    config: &AppConfig,
) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let delegated: Vec<Uuid> = delegations
        .iter()
        .map(|d| d.permission_point_id)
        .collect();

    let claims = Claims {
        sub: user.id,
        role_id: role.id,
        role_name: role.name.clone(),
        data_scope: format!("{:?}", role.data_scope).to_lowercase(),
        scope_value: role.scope_value.clone(),
        department: user.department.clone(),
        location: user.location.clone(),
        delegated_permissions: delegated,
        exp: now + config.jwt_access_ttl_secs,
        iat: now,
        token_type: "access".into(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Token encoding failed: {}", e)))
}

pub fn issue_refresh_token(user_id: Uuid, config: &AppConfig) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id,
        role_id: Uuid::nil(),
        role_name: String::new(),
        data_scope: String::new(),
        scope_value: None,
        department: None,
        location: None,
        delegated_permissions: vec![],
        exp: now + config.jwt_refresh_ttl_secs,
        iat: now,
        token_type: "refresh".into(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Token encoding failed: {}", e)))
}

pub fn decode_token(token: &str, config: &AppConfig) -> Result<Claims, AppError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

    Ok(data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::role::{DataScope, Role};
    use crate::models::user::User;
    use chrono::Utc;

    fn test_config(secret: &str) -> AppConfig {
        AppConfig {
            database_url: "unused".into(),
            jwt_secret: secret.into(),
            jwt_access_ttl_secs: 3600,
            jwt_refresh_ttl_secs: 86400,
            field_encryption_key: [0u8; 32],
            lockout_threshold: 5,
            lockout_duration_secs: 900,
        }
    }

    fn test_user() -> User {
        User {
            id: Uuid::nil(),
            username: "testuser".into(),
            password_hash_enc: vec![],
            gov_id_enc: None,
            gov_id_last4: None,
            role_id: Uuid::nil(),
            department: Some("sales".into()),
            location: Some("store-1".into()),
            is_active: true,
            failed_attempts: 0,
            locked_until: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn test_role() -> Role {
        Role {
            id: Uuid::nil(),
            name: "TestRole".into(),
            description: None,
            data_scope: DataScope::Department,
            scope_value: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_issue_and_decode_access_token() {
        let config = test_config("test-secret-key");
        let user = test_user();
        let role = test_role();
        let token = issue_access_token(&user, &role, &[], &config)
            .expect("issue_access_token should succeed");
        let claims = decode_token(&token, &config)
            .expect("decode_token should succeed");
        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.role_id, role.id);
        assert_eq!(claims.role_name, role.name);
        assert_eq!(claims.token_type, "access");
        assert_eq!(claims.department, user.department);
        assert_eq!(claims.location, user.location);
    }

    #[test]
    fn test_refresh_token_type() {
        let config = test_config("test-secret-key");
        let user_id = Uuid::nil();
        let token = issue_refresh_token(user_id, &config)
            .expect("issue_refresh_token should succeed");
        let claims = decode_token(&token, &config)
            .expect("decode_token should succeed");
        assert_eq!(claims.token_type, "refresh");
        assert_eq!(claims.sub, user_id);
    }

    #[test]
    fn test_decode_wrong_secret_fails() {
        let config_encode = test_config("secret1");
        let config_decode = test_config("secret2");
        let user = test_user();
        let role = test_role();
        let token = issue_access_token(&user, &role, &[], &config_encode)
            .expect("issue_access_token should succeed");
        let result = decode_token(&token, &config_decode);
        assert!(result.is_err(), "Decoding with wrong secret should fail");
    }
}
