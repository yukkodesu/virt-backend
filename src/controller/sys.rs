use crate::{
    middleware::authenticate::JWT,
    virt::{VirtCommand, VirtCommandType, VirtConnect},
};
use rocket::{http::Status, response::content, State};

#[get("/utilization/get")]
pub fn get_sys_utilization(_jwt: JWT, conn: &State<VirtConnect>) -> (Status, content::RawJson<String>) {
    let conn = conn as &VirtConnect;
    conn.tx
        .send(VirtCommand::create(VirtCommandType::SysInfo))
        .unwrap();
    match conn.rx.lock().unwrap().recv() {
        Ok(output) => match output {
            Ok(res) => (Status::Ok, content::RawJson(res)),
            Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
        },
        Err(e) => (Status::InternalServerError, content::RawJson(e.to_string())),
    }
}
