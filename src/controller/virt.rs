use rocket::{
    data::{Data, ToByteUnit},
    http::Status,
    response::content,
    serde::json::Json,
    State,
};

use crate::{
    middleware::authenticate::JWT,
    virt::{shell, VirtCommand, VirtCommandType, VirtConnect},
};

#[get("/hello")]
pub fn hello(_jwt: JWT) -> String {
    String::from("hello")
}

#[get("/list-all")]
pub fn list_all(_jwt: JWT, conn: &State<VirtConnect>) -> (Status, content::RawJson<String>) {
    let conn = conn as &VirtConnect;
    conn.tx
        .send(VirtCommand::create(VirtCommandType::ListAll))
        .unwrap();
    if let Ok(res) = conn.rx.lock().unwrap().recv() {
        return (Status::Ok, content::RawJson(res));
    }
    (
        Status::InternalServerError,
        content::RawJson(String::from("Error listing all domains")),
    )
}

#[post("/list-snapshot", format = "application/json", data = "<dom_names>")]
pub fn list_snapshot(
    _jwt: JWT,
    conn: &State<VirtConnect>,
    dom_names: Json<Vec<String>>,
) -> (Status, content::RawJson<String>) {
    let conn = conn as &VirtConnect;
    let dom_names: Vec<String> = dom_names.0.into_iter().collect();
    conn.tx
        .send(VirtCommand::create_with_params(
            VirtCommandType::ListSnapshot,
            dom_names,
        ))
        .unwrap();
    if let Ok(res) = conn.rx.lock().unwrap().recv() {
        return (Status::Ok, content::RawJson(res));
    }
    (
        Status::InternalServerError,
        content::RawJson(String::from("Error listing snapshots")),
    )
}

#[post(
    "/list-snapshot-tree",
    format = "application/json",
    data = "<dom_name>"
)]
pub fn list_snapshot_tree(
    _jwt: JWT,
    conn: &State<VirtConnect>,
    dom_name: Json<String>,
) -> (Status, content::RawJson<String>) {
    let conn = conn as &VirtConnect;
    conn.tx
        .send(VirtCommand::create_with_params(
            VirtCommandType::ListSnapshotTree,
            vec![dom_name.0],
        ))
        .unwrap();
    if let Ok(res) = conn.rx.lock().unwrap().recv() {
        return (Status::Ok, content::RawJson(res));
    }
    (
        Status::InternalServerError,
        content::RawJson(String::from("Error listing snapshot tree")),
    )
}
#[post("/snapshot-current", format = "application/json", data = "<dom_names>")]
pub fn snapshot_current(
    _jwt: JWT,
    conn: &State<VirtConnect>,
    dom_names: Json<Vec<String>>,
) -> (Status, content::RawJson<String>) {
    let conn = conn as &VirtConnect;
    let dom_names: Vec<String> = dom_names.0.into_iter().collect();
    conn.tx
        .send(VirtCommand::create_with_params(
            VirtCommandType::SnapShotCurrent,
            dom_names,
        ))
        .unwrap();
    if let Ok(res) = conn.rx.lock().unwrap().recv() {
        return (Status::Ok, content::RawJson(res));
    }
    (
        Status::InternalServerError,
        content::RawJson(String::from("Error getting snapshot current")),
    )
}

#[post("/create-snapshot", format = "application/json", data = "<configure>")]
pub fn create_snapshot(
    _jwt: JWT,
    configure: Json<shell::SnapShotConfig>,
) -> (Status, content::RawJson<String>) {
    match shell::create_snapshot(configure.0) {
        Ok(output) => (Status::Ok, content::RawJson(output)),
        Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
    }
}

#[post("/delete-snapshot", format = "application/json", data = "<configure>")]
pub fn delete_snapshot(
    _jwt: JWT,
    configure: Json<shell::SnapShotConfig>,
) -> (Status, content::RawJson<String>) {
    match shell::delete_snapshot(configure.0) {
        Ok(output) => (Status::Ok, content::RawJson(output)),
        Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
    }
}

#[post("/upload-iso", data = "<isofile>")]
pub async fn upload_iso(isofile: Data<'_>) -> (Status, String) {
    match isofile.open(8.gigabytes()).into_file("./test.iso").await {
        Ok(_) => (Status::Ok, "".to_string()),
        Err(e) => (Status::InsufficientStorage, e.to_string()),
    }
}
