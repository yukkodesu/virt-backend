use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode,
    errors::{Error, ErrorKind},
    DecodingKey, EncodingKey, Header, Validation,
};
use rocket::{
    http::Status,
    request::{self, FromRequest, Outcome},
    serde::{Deserialize, Serialize},
    Request,
};
use std::env;

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: i32,
    iat: usize,
    exp: usize,
}

#[derive(Debug)]
pub struct JWT {
    pub claims: Claims,
}

pub fn create_jwt(id: i32) -> Result<String, Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set.");
    let now = Utc::now();
    let issue_at_time = now.timestamp();
    let expiration = now
        .checked_add_signed(Duration::hours(12))
        .expect("Invalid timestamp")
        .timestamp();
    let claims = Claims {
        sub: id,
        iat: issue_at_time as usize,
        exp: expiration as usize,
    };
    let header = Header::new(jsonwebtoken::Algorithm::HS512);
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn decode_jwt(token: &str) -> Result<Claims, Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(jsonwebtoken::Algorithm::HS512),
    ) {
        Ok(token) => Ok(token.claims),
        Err(err) => Err(err),
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JWT {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match req.headers().get_one("authorization") {
            None => Outcome::Error((
                Status::Unauthorized,
                String::from("Error validating JWT token - No token provided"),
            )),
            Some(token) => match decode_jwt(token) {
                Ok(claims) => Outcome::Success(JWT { claims }),
                Err(e) => Outcome::Error((
                    Status::Unauthorized,
                    String::from(match e.kind() {
                        ErrorKind::ExpiredSignature => {
                            "Error validating JWT token - ExpiredSignature"
                        }
                        ErrorKind::InvalidToken => "Error validating JWT token - InvalidToken",
                        _ => "",
                    }),
                )),
            },
        }
    }
}