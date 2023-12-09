use crate::middleware::authenticate::JWT;

#[get("/hello")]
pub fn hello(_jwt: JWT) -> String {
    String::from("hello")
}
