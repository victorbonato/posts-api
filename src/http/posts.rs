use futures::TryStreamExt;

use crate::http::{Error, Result};
use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{extractor::AuthUser, ApiContext};

pub(crate) fn router() -> Router<ApiContext> {
    Router::new()
        .route("/api/posts", get(list_posts))
        .route("/api/:username/posts", get(get_user_posts))
        .route("/api/posts", post(add_post))
        .route(
            "/api/posts/:post_id",
            delete(delete_post).patch(update_post),
        )
}

#[derive(Serialize, Deserialize)]
struct PostBody<T = Post> {
    post: T,
}

#[derive(Serialize)]
struct MultiplePostsBody<T: Serialize> {
    posts: Vec<T>,
}

#[derive(Deserialize)]
struct NewPost {
    title: String,
    content: String,
}

#[derive(Serialize)]
struct PostForFeed {
    user: String,
    title: String,
    content: String,
}

#[derive(serde::Deserialize, Default, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
struct UpdatePost {
    user_id: Option<Uuid>,
    title: Option<String>,
    content: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Post {
    post_id: i64,
    user_id: Uuid,
    created_at: OffsetDateTime,
    updated_at: Option<OffsetDateTime>,
    title: String,
    content: String,
}

async fn list_posts(ctx: State<ApiContext>) -> Result<Json<MultiplePostsBody<PostForFeed>>> {
    let posts = sqlx::query_as!(
        PostForFeed,
        r#"select username as user, title, content from "user" inner join "post" on "user".user_id = "post".user_id "#,
    )
    .fetch_all(&ctx.db)
    .await?;

    Ok(Json(MultiplePostsBody { posts }))
}

async fn get_user_posts(
    ctx: State<ApiContext>,
    Path(username): Path<String>,
) -> Result<Json<MultiplePostsBody<Post>>> {
    let user_id = get_post_id_by_username(&ctx, username).await?;

    let posts = sqlx::query_as!(
        Post,
        r#"select * from post where user_id = $1 order by created_at"#,
        user_id
    )
    .fetch(&ctx.db)
    .try_collect()
    .await?;

    Ok(Json(MultiplePostsBody { posts }))
}

async fn add_post(
    auth_user: AuthUser,
    ctx: State<ApiContext>,
    Json(req): Json<PostBody<NewPost>>,
) -> Result<Json<PostBody<Post>>> {
    let post = sqlx::query_as!(
        Post,
        r#"
            insert into "post" (user_id, title, content)
            values ($1, $2, $3)
            returning *
        "#,
        auth_user.user_id,
        req.post.title,
        req.post.content,
    )
    .fetch_one(&ctx.db)
    .await?;

    Ok(Json(PostBody { post }))
}

async fn delete_post(
    auth_user: AuthUser,
    ctx: State<ApiContext>,
    Path(post_id): Path<i64>,
) -> Result<()> {
    let result = sqlx::query!(
        r#"
            with deleted_post as (
                delete from "post"
                where
                    post_id = $1
                    and user_id = $2
                returning 1
            )
            select
            exists(
                select 1 from "post" where post_id = $1
            ) "existed!",
            exists(select 1 from deleted_post) "deleted!"
        "#,
        post_id,
        auth_user.user_id
    )
    .fetch_one(&ctx.db)
    .await?;

    if result.deleted {
        Ok(())
    } else if result.existed {
        Err(Error::Forbidden)
    } else {
        Err(Error::NotFound)
    }
}

async fn update_post(
    auth_user: AuthUser,
    ctx: State<ApiContext>,
    Path(post_id): Path<i64>,
    Json(req): Json<PostBody<UpdatePost>>,
) -> Result<Json<PostBody<Post>>> {
    // Check if the post exists and if the user is the owner
    let post_check = sqlx::query!(
        r#"
        SELECT
            EXISTS(SELECT 1 FROM "post" WHERE post_id = $1) AS "post_exists!",
            EXISTS(SELECT 1 FROM "post" WHERE post_id = $1 AND user_id = $2) AS "user_is_owner!"
        FROM "post"
        WHERE post_id = $1
        LIMIT 1
        "#,
        post_id,
        auth_user.user_id,
    )
    .fetch_optional(&ctx.db)
    .await?
    .ok_or(Error::NotFound)?;

    // Handle unauthorized access
    if !post_check.user_is_owner {
        return Err(Error::Forbidden);
    }

    // Proceed with the update if the post exists and the user is the owner
    let updated_post = sqlx::query_as!(
        Post,
        r#"
        UPDATE "post"
        SET
            user_id = COALESCE($1, user_id),
            title = COALESCE($2, title),
            content = COALESCE($3, content)
        WHERE post_id = $4
        RETURNING *
        "#,
        req.post.user_id,
        req.post.title,
        req.post.content,
        post_id,
    )
    .fetch_one(&ctx.db)
    .await?;

    // Return the updated post
    Ok(Json(PostBody { post: updated_post }))
}

async fn get_post_id_by_username(ctx: &State<ApiContext>, username: String) -> Result<Uuid> {
    let user_id = sqlx::query_scalar!(
        r#"
            select user_id from "user" where username = $1
        "#,
        username
    )
    .fetch_optional(&ctx.db)
    .await?
    .ok_or(Error::NotFound)?;

    Ok(user_id)
}

async fn load_post_by_id(ctx: &State<ApiContext>, post_id: i64) -> Result<Post> {
    let post = sqlx::query_as!(Post, r#"select * from "post" where post_id = $1 "#, post_id)
        .fetch_optional(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;

    Ok(post)
}
