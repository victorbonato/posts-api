use async_trait::async_trait;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use sqlx::types::Uuid;
use time::OffsetDateTime;

use super::ApiContext;
use crate::http::Error;

const DEFAULT_SESSION_LENGTH: time::Duration = time::Duration::weeks(2);

pub struct AuthUser {
    pub user_id: Uuid,
}

pub struct MaybeAuthUser(pub Option<AuthUser>);

#[derive(serde::Serialize, serde::Deserialize)]
struct AuthUserClaims {
    user_id: Uuid,
    exp: usize,
}

impl AuthUser {
    pub(in crate::http) fn to_jwt(&self, ctx: &ApiContext) -> String {
        let claims = AuthUserClaims {
            user_id: self.user_id,
            exp: (OffsetDateTime::now_utc() + DEFAULT_SESSION_LENGTH).unix_timestamp() as usize,
        };

        encode(
            &Header::new(Algorithm::HS384),
            &claims,
            &EncodingKey::from_secret(ctx.config.jwt_secret.as_ref()),
        )
        .expect("JWT encoding should be infallible")
    }

    fn from_jwt(ctx: &ApiContext, auth_header: Bearer) -> Result<Self, Error> {
        let token = auth_header.token();

        let validation = Validation::new(Algorithm::HS384);
        let token_data = decode::<AuthUserClaims>(
            token,
            &DecodingKey::from_secret(ctx.config.jwt_secret.as_ref()),
            &validation,
        )
        .map_err(|e| {
            log::debug!("JWT failed to verify: {}", e);
            Error::Unauthorized
        })?;

        let claims = token_data.claims;

        if claims.exp < OffsetDateTime::now_utc().unix_timestamp() as usize {
            log::debug!("token expired");
            return Err(Error::Unauthorized);
        }

        Ok(Self {
            user_id: claims.user_id,
        })
    }
}

impl MaybeAuthUser {
    pub fn user_id(&self) -> Option<Uuid> {
        self.0.as_ref().map(|auth_user| auth_user.user_id)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    ApiContext: FromRef<S>,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ctx = ApiContext::from_ref(state);

        let bearer = extract_bearer(parts, state).await?;

        Self::from_jwt(&ctx, bearer)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for MaybeAuthUser
where
    S: Send + Sync,
    ApiContext: FromRef<S>,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ctx = ApiContext::from_ref(state);

        let maybe_bearer = extract_bearer(parts, state).await.ok();

        Ok(Self(
            maybe_bearer
                .map(|bearer| AuthUser::from_jwt(&ctx, bearer))
                .transpose()?,
        ))
    }
}

async fn extract_bearer<S: Send + Sync>(parts: &mut Parts, state: &S) -> Result<Bearer, Error> {
    let TypedHeader(Authorization(bearer)) =
        TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::Unauthorized)?;
    Ok(bearer)
}
