use std::sync::Arc;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, HeaderMap, StatusCode},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::Role;

#[derive(Clone)]
pub struct JwtKeys {
    pub enc: Arc<EncodingKey>,
    pub dec: Arc<DecodingKey>,
}

impl JwtKeys {
    pub fn from_secret(secret: &str) -> Self {
        Self {
            enc: Arc::new(EncodingKey::from_secret(secret.as_bytes())),
            dec: Arc::new(DecodingKey::from_secret(secret.as_bytes())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub role: String,
    pub csrf: String,
    pub exp: usize,
    pub typ: String,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
    pub role: String,
    pub csrf: String,
}

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

pub fn create_token_pair(
    user_id: Uuid,
    email: &str,
    role: Role,
    keys: &JwtKeys,
) -> anyhow::Result<(String, String, String)> {
    let csrf = Uuid::new_v4().to_string();

    let access = Claims {
        sub: user_id,
        email: email.to_owned(),
        role: role.as_str().to_owned(),
        csrf: csrf.clone(),
        exp: (Utc::now() + Duration::minutes(30)).timestamp() as usize,
        typ: "access".to_owned(),
    };

    let refresh = Claims {
        sub: user_id,
        email: email.to_owned(),
        role: role.as_str().to_owned(),
        csrf: csrf.clone(),
        exp: (Utc::now() + Duration::days(14)).timestamp() as usize,
        typ: "refresh".to_owned(),
    };

    Ok((
        encode(&Header::default(), &access, &keys.enc)?,
        encode(&Header::default(), &refresh, &keys.enc)?,
        csrf,
    ))
}

pub fn decode_token(token: &str, keys: &JwtKeys) -> anyhow::Result<Claims> {
    let data = decode::<Claims>(token, &keys.dec, &Validation::default())?;
    Ok(data.claims)
}

pub fn require_csrf(headers: &HeaderMap, csrf: &str) -> Result<(), (StatusCode, String)> {
    let provided = headers
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if provided != csrf {
        return Err((StatusCode::FORBIDDEN, "Invalid CSRF token".to_owned()));
    }

    Ok(())
}

#[axum::async_trait]
impl FromRequestParts<crate::AppState> for AuthUser {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let token = auth
            .strip_prefix("Bearer ")
            .ok_or((StatusCode::UNAUTHORIZED, "Missing bearer token".to_owned()))?;

        let claims = decode_token(token, &state.jwt_keys)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_owned()))?;

        if claims.typ != "access" {
            return Err((StatusCode::UNAUTHORIZED, "Invalid token type".to_owned()));
        }

        Ok(Self {
            user_id: claims.sub,
            email: claims.email,
            role: claims.role,
            csrf: claims.csrf,
        })
    }
}
