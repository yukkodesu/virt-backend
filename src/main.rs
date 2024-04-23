#[macro_use]
extern crate rocket;

mod controller;
mod db;
mod middleware;
mod scheduler;
#[cfg(test)]
mod test;
mod virt;

use controller::{account::*, snapshot::*, sys::get_sys_utilization, virt::*, vnc::*};
use db::init;
use dotenvy::dotenv;
use futures::executor::block_on;
use scheduler::SchedConnect;
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

    let sched_conn = SchedConnect::new().await;

    rocket::build()
        .manage(db)
        .manage(virt_conn)
        .manage(sched_conn)
        .mount("/api/v1/account", routes![login_handler, regist_handler])
        .mount("/api/v1/sys", routes![get_sys_utilization])
        .mount(
            "/api/v1/virt",
            routes![list_domains, set_domain_state, upload_iso],
        )
        .mount(
            "/api/v1/snapshot",
            routes![
                list_snapshot,
                list_snapshot_tree,
                edit_snapshot,
                set_current_snapshot,
                clone_snapshot_as_vm,
                create_snapshot,
                delete_snapshot,
                add_sched_task,
                delete_sched_task,
            ],
        )
        .mount("/api/v1/vnc", routes![vnc_connect, get_vnc_display_config])
    // .attach(CORS)
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    build().await.launch().await?;
    Ok(())
}
