use crate::db::entity::{prelude::*, *};
use crate::middleware::authenticate::create_jwt;
use bcrypt::{hash, verify, DEFAULT_COST};
use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use rocket::time::{OffsetDateTime, Duration};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, ActiveValue};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserJson {
    pub username: String,
    pub password: String,
}

#[derive(Responder, Debug)]
pub enum NetworkResponse {
    #[response(status = 200)]
    Success(String),
    #[response(status = 400)]
    BadRequest(String),
    #[response(status = 401)]
    Unauthorized(String),
    #[response(status = 500)]
    InternalError(String),
}

#[post("/login", format = "application/json", data = "<req_user>")]
pub async fn login_handler(
    req_user: Json<UserJson>,
    db: &State<DatabaseConnection>,
    cookies: &CookieJar<'_>
) -> NetworkResponse {
    let db = db as &DatabaseConnection;
    let user_db = match User::find()
        .filter(user::Column::Username.eq(&req_user.username))
        .one(db)
        .await
    {
        Ok(Some(v)) => v,
        _ => return NetworkResponse::Unauthorized(String::from("Password or Username is wrong")),
    };

    let verify = verify(&req_user.password, &user_db.password).expect("Password verify error");
    if !verify {
        return NetworkResponse::Unauthorized(String::from("Password or Username is wrong"));
    }
    let token = create_jwt(user_db.id).expect("create jwt error");

    let expire_time = OffsetDateTime::now_utc() + Duration::days(1);
    let mut token_cookie = Cookie::new("authorization", token.clone());
    token_cookie.set_expires(expire_time);
    cookies.add(token_cookie);
    NetworkResponse::Success(token)
}


#[post("/regist", format = "application/json", data = "<req_user>")]
pub async fn regist_handler(
    req_user: Json<UserJson>,
    db: &State<DatabaseConnection>,
) -> NetworkResponse {
    let db = db as &DatabaseConnection;
    match User::find()
        .filter(user::Column::Username.eq(&req_user.username))
        .one(db)
        .await
    {
        Ok(Some(_)) => return NetworkResponse::Unauthorized(String::from("User already exists")),
        _ => (),
    }

    let hashed = hash(&req_user.password, DEFAULT_COST).expect("Password verify error");
    if let Err(err) = User::insert(user::ActiveModel{
        username: ActiveValue::set(req_user.username.clone()),
        password: ActiveValue::Set(hashed),
        ..Default::default()
    }).exec(db).await {
        return NetworkResponse::InternalError(err.to_string());
    }
    NetworkResponse::Success(String::from(""))
}