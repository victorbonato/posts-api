use anyhow::{anyhow, Context};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash};
use axum::routing::{get, post};
use axum::{extract::State, Json, Router};

use crate::http::extractor::AuthUser;
use crate::http::{Error, Result};

use super::{ApiContext, ResultExt};

pub(crate) fn router() -> Router<ApiContext> {
    Router::new()
        .route("/api/users", post(create_user))
        .route("/api/users/login", post(login_user))
        .route("/api/user", get(get_current_user).patch(update_user))
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UserBody<T> {
    user: T,
}

#[derive(serde::Deserialize)]
struct NewUser {
    username: String,
    password: String,
}

#[derive(serde::Deserialize)]
struct LoginUser {
    username: String,
    password: String,
}

#[derive(serde::Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
struct UpdateUser {
    username: Option<String>,
    password: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct User {
    token: String,
    username: String,
}

async fn create_user(
    ctx: State<ApiContext>,
    Json(req): Json<UserBody<NewUser>>,
) -> Result<Json<UserBody<User>>> {
    let password_hash = hash_password(req.user.password).await?;

    let user_id = sqlx::query_scalar!(
        r#"insert into "user" (username, password_hash) values ($1, $2) returning user_id"#,
        req.user.username,
        password_hash
    )
    .fetch_one(&ctx.db)
    .await
    .on_constraint("user_username_key", |_| {
        Error::unprocessable_entity([("username", "username taken")])
    })?;

    Ok(Json(UserBody {
        user: User {
            token: AuthUser { user_id }.to_jwt(&ctx),
            username: req.user.username,
        },
    }))
}

async fn login_user(
    ctx: State<ApiContext>,
    Json(req): Json<UserBody<LoginUser>>,
) -> Result<Json<UserBody<User>>> {
    let user = sqlx::query!(
        r#"
            select user_id, username, password_hash from "user" where username = $1
        "#,
        req.user.username
    )
    .fetch_optional(&ctx.db)
    .await?
    .ok_or_else(|| Error::unprocessable_entity([("email", "does not exist")]))?;

    verify_password(req.user.password, user.password_hash).await?;

    Ok(Json(UserBody {
        user: User {
            token: AuthUser {
                user_id: user.user_id,
            }
            .to_jwt(&ctx),
            username: req.user.username,
        },
    }))
}

async fn get_current_user(
    auth_user: AuthUser,
    ctx: State<ApiContext>,
) -> Result<Json<UserBody<User>>> {
    let user = sqlx::query!(
        r#"select username from "user" where user_id = $1"#,
        auth_user.user_id
    )
    .fetch_one(&ctx.db)
    .await?;

    Ok(Json(UserBody {
        user: User {
            token: auth_user.to_jwt(&ctx),
            username: user.username,
        },
    }))
}

async fn update_user(
    auth_user: AuthUser,
    ctx: State<ApiContext>,
    Json(req): Json<UserBody<UpdateUser>>,
) -> Result<Json<UserBody<User>>> {
    if req.user == UpdateUser::default() {
        return get_current_user(auth_user, ctx).await;
    }

    let password_hash = if let Some(password) = req.user.password {
        Some(hash_password(password).await?)
    } else {
        None
    };

    let user = sqlx::query!(
        r#"
            update "user"
            set username = coalesce($1, "user".username),
                password_hash = coalesce($2, "user".password_hash)
            where user_id = $3
            returning username
        "#,
        req.user.username,
        password_hash,
        auth_user.user_id
    )
    .fetch_one(&ctx.db)
    .await
    .on_constraint("user_username_key", |_| {
        Error::unprocessable_entity([("username", "username taken")])
    })?;

    Ok(Json(UserBody {
        user: User {
            token: auth_user.to_jwt(&ctx),
            username: user.username,
        },
    }))
}

async fn hash_password(password: String) -> Result<String> {
    tokio::task::spawn_blocking(move || -> Result<String> {
        let salt = SaltString::generate(rand::thread_rng());
        Ok(
            PasswordHash::generate(Argon2::default(), password, salt.as_salt())
                .map_err(|e| anyhow!("failed to generate password hash: {}", e))?
                .to_string(),
        )
    })
    .await
    .context("panic in generating password hash")?
}

async fn verify_password(password: String, password_hash: String) -> Result<()> {
    tokio::task::spawn_blocking(move || -> Result<()> {
        let hash = PasswordHash::new(&password_hash)
            .map_err(|e| anyhow!("invalid password hash: {}", e))?;

        hash.verify_password(&[&Argon2::default()], password)
            .map_err(|e| match e {
                argon2::password_hash::Error::Password => Error::Unauthorized,
                _ => anyhow::anyhow!("failed to verify  password hash: {}", e).into(),
            })
    })
    .await
    .context("panic in verifying password hash")?
}
