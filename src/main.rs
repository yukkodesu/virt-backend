#[macro_use]
extern crate rocket;

mod controller;
mod db;
mod middleware;
#[cfg(test)]
mod test;
mod virt;

use controller::{account::*, sysinfo::get_sysinfo, virt::*};
use db::init;
use dotenvy::dotenv;
use futures::executor::block_on;
use middleware::cors::CORS;
use std::env;
use virt::VirtConnect;

async fn build() -> rocket::Rocket<rocket::Build> {
    dotenv().expect(".env file not found");
    let db = match block_on(init(
        &env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
    )) {
        Ok(db) => db,
        Err(err) => {
            panic!("{}", err);
        }
    };

    let virt_conn = VirtConnect::new();

    rocket::build()
        .manage(db)
        .manage(virt_conn)
        .mount(
            "/api",
            routes![
                login_handler,
                regist_handler,
                hello,
                list_all,
                list_snapshot,
                list_snapshot_tree,
                edit_snapshot,
                set_current_snapshot,
                clone_snapshot_as_vm,
                get_sysinfo,
                create_snapshot,
                delete_snapshot,
                upload_iso,
                alt_vm_state,
            ],
        )
        .attach(CORS)
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    build().await.launch().await?;
    Ok(())
}
