use rocket::{http::Status, response::content, State, serde::json::Json};

use crate::{
    middleware::authenticate::JWT,
    virt::{VirtCommand, VirtConnect},
};

#[get("/hello")]
pub fn hello(_jwt: JWT) -> String {
    String::from("hello")
}

#[get("/list-all")]
pub fn list_all(_jwt: JWT, conn: &State<VirtConnect>) -> (Status, content::RawJson<String>) {
    let conn = conn as &VirtConnect;
    conn.tx
        .send(VirtCommand::create("ListAll"))
        .unwrap();
    if let Ok(res) = conn.rx.lock().unwrap().recv() {
        return (Status::Ok, content::RawJson(res));
    }
    (
        Status::InternalServerError,
        content::RawJson(String::from("Error listing all domains")),
    )
}

#[post("/list-snapshot",format = "application/json", data = "<dom_names>")]
pub fn list_snapshot(
    _jwt: JWT,
    conn: &State<VirtConnect>,
    dom_names: Json<Vec<String>>
) -> (Status, content::RawJson<String>) {
    let conn = conn as &VirtConnect;
    let dom_names = dom_names.0.into_iter().collect();
    conn.tx.send(VirtCommand::create_with_params("ListSnapshot", dom_names)).unwrap();
    if let Ok(res) = conn.rx.lock().unwrap().recv() {
        return (Status::Ok, content::RawJson(res));
    }
    (
        Status::InternalServerError,
        content::RawJson(String::from("Error listing all domains")),
    )
}
