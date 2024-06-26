use anyhow::Context;
use axum::extract::{Query, State};
use reqwest::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::routes::AppState;

use super::UserError;

#[derive(Deserialize, IntoParams)]
pub struct SignUpVerificationQuery {
    token: String,
}

#[utoipa::path(
    put,
    tag="user",
    path="/user/sign-up-verification",
    params(
        SignUpVerificationQuery
    ),
    responses(
        (status = StatusCode::ACCEPTED, description = "user successfully registered"),
        (status = StatusCode::INTERNAL_SERVER_ERROR, description = "internal server error")
    )
)]
#[tracing::instrument(skip_all)]
/// verify user
///
/// changes user's `verified` status to true
pub async fn sign_up_verification(
    State(app): State<AppState>,
    Query(query): Query<SignUpVerificationQuery>,
) -> Result<StatusCode, UserError> {
    let pool = app.pool;

    let user_id = get_user_id_from_token(&pool, query.token)
        .await
        .context("failed to retrieve user_id associated with provided token")?
        .ok_or(UserError::UnknownToken)?;

    confirm_user(&pool, user_id)
        .await
        .context("failed to update user verified status to true")?;

    Ok(StatusCode::ACCEPTED)
}

async fn confirm_user(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "update users set verified = true where user_id = $1",
        user_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn get_user_id_from_token(pool: &PgPool, token: String) -> Result<Option<Uuid>, sqlx::Error> {
    let res = sqlx::query!(
        "select user_id from verification_tokens where verification_token = $1",
        token
    )
    .fetch_optional(pool)
    .await?;
    Ok(res.map(|x| x.user_id))
}
