use axum::{
    extract::Extension,
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{CreateUser, User};

pub async fn health() -> &'static str {
    "OK"
}

pub async fn create_user(
    Extension(pool): Extension<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), (StatusCode, String)> {
    let id = Uuid::new_v4();

    let rec = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, name, email)
        VALUES ($1, $2, $3)
        RETURNING id, name, email
        "#,
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&payload.email)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(rec)))
}

pub async fn list_users(
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<User>>, (StatusCode, String)> {
    let recs = sqlx::query_as::<_, User>("SELECT id, name, email FROM users")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(recs))
}
