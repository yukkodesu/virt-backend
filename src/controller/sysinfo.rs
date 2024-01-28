use crate::{
    middleware::authenticate::JWT,
    virt::{VirtCommand, VirtConnect},
};
use rocket::{http::Status, response::content, State};

#[get("/sysinfo")]
pub fn get_sysinfo(_jwt: JWT, conn: &State<VirtConnect>) -> (Status, content::RawJson<String>) {
    let conn = conn as &VirtConnect;
    conn.tx.send(VirtCommand::create("SysInfo")).unwrap();
    if let Ok(res) = conn.rx.lock().unwrap().recv() {
        return (Status::Ok, content::RawJson(res));
    }
    (
        Status::InternalServerError,
        content::RawJson(String::from("Error get system info")),
    )
}
