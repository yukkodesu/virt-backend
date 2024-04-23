use rocket::{
    data::{Data, ToByteUnit},
    http::Status,
    response::content,
    serde::json::Json,
    State,
};

use crate::{
    middleware::authenticate::JWT,
    virt::{shell, AltDomStateCommand, VirtCommand, VirtCommandType, VirtConnect},
};

#[get("/list")]
pub fn list_domains(_jwt: JWT, conn: &State<VirtConnect>) -> (Status, content::RawJson<String>) {
    let conn = conn as &VirtConnect;
    if let Err(e) = conn.tx.send(VirtCommand::create(VirtCommandType::ListAll)) {
        return (
            Status::InternalServerError,
            content::RawJson(
                String::from("Error sending VirtCommand to LibVirt Thread:") + &e.to_string(),
            ),
        );
    }
    match conn.rx.lock().unwrap().recv() {
        Ok(output) => match output {
            Ok(res) => (Status::Ok, content::RawJson(res)),
            Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
        },
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

#[post("/set-state", data = "<config>")]
pub async fn set_domain_state(
    _jwt: JWT,
    config: Json<AltDomStateCommand>,
) -> (Status, content::RawJson<String>) {
    match shell::alt_vm_state(config.0) {
        Ok(output) => (Status::Ok, content::RawJson(output)),
        Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
    }
}
