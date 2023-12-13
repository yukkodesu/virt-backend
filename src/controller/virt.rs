use rocket::State;

use crate::{middleware::authenticate::JWT, virt::VirtConnect};

#[get("/hello")]
pub fn hello(_jwt: JWT) -> String {
    String::from("hello")
}

#[get("/list-all")]
pub fn list_all(conn: &State<VirtConnect>) -> String {
    let conn = conn as &VirtConnect;
    conn.tx.send(String::from("ListAll")).unwrap();
    if let Ok(res) = conn.rx.lock().unwrap().recv() {
        return res;
    }
    String::from("Error")
}
