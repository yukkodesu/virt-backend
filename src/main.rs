#[macro_use]
extern crate rocket;

mod controller;
mod db;
mod middleware;

use controller::{account::*, virt::*};
use db::init;
use dotenvy::dotenv;
use futures::executor::block_on;
use std::env;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv().expect(".env file not found");
    let db = match block_on(init(
        &env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
    )) {
        Ok(db) => db,
        Err(err) => {
            panic!("{}", err);
        }
    };

    let _ = rocket::build()
        .manage(db)
        .mount("/api", routes![login_handler, regist_handler, hello])
        .launch()
        .await?;

    Ok(())
}
