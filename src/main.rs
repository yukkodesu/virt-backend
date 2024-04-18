#[macro_use]
extern crate rocket;

mod controller;
mod db;
mod middleware;
mod scheduler;
mod virt;
#[cfg(test)]
mod test;

use controller::{account::*, sysinfo::get_sysinfo, virt::*, vnc::*};
use db::init;
use dotenvy::dotenv;
use futures::executor::block_on;
use middleware::cors::CORS;
use scheduler::Scheduler;
use std::env;
use virt::VirtConnect;

async fn build() -> rocket::Rocket<rocket::Build> {
    dotenv().expect(".env file not found");
    let db = match block_on(init(
        &env::var("DATABASE_URL").expect("DATABASE_URL must be set."),
    )) {
        Ok(db) => db,
        Err(err) => panic!("{}", err),
    };

    let virt_conn = VirtConnect::new();

    let scheduler = match Scheduler::new().await {
        Ok(v) => v,
        Err(e) => panic!("{}", e),
    };

    rocket::build()
        .manage(db)
        .manage(virt_conn)
        .manage(scheduler)
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
                vnc_connect,
                get_vnc_display_config
            ],
        )
        .attach(CORS)
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    build().await.launch().await?;
    Ok(())
}
