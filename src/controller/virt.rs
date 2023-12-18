use rocket::{State, response::content};

use crate::{middleware::authenticate::JWT, virt::VirtConnect};

#[get("/hello")]
pub fn hello(_jwt: JWT) -> String {
    String::from("hello")
}

#[get("/list-all")]
pub fn list_all(conn: &State<VirtConnect>) -> content::RawJson<String> {
    let conn = conn as &VirtConnect;
    conn.tx.send(String::from("ListAll")).unwrap();
    if let Ok(res) = conn.rx.lock().unwrap().recv() {
        return content::RawJson(res);
    }
    content::RawJson(String::from("Error"))
}
