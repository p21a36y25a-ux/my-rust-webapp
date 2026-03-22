use sqlx::PgPool;

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    if let Err(e) = sqlx::migrate!("./migrations").run(pool).await {
        tracing::error!("migrations failed: {}", e);
        return Err(e);
    }

    Ok(())
}
