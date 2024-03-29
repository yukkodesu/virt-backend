use std::env;

use rocket::http::ContentType;
use rocket::local::asynchronous::Client;
use serde_json::json;

mod virt;

pub async fn get_auth(client: &Client) -> String {
    let admin_secret = env::var("ADMIN_SECRET").unwrap();
    let body = json!({"username":"admin", "password": admin_secret}).to_string();
    println!("{}", body);
    let mut login = client.post(uri!("/api/login")).body(body);
    login.add_header(ContentType::JSON);
    let login_response = login.dispatch().await;
    let auth = login_response.cookies().get("authorization").unwrap();
    auth.value().to_string()
}
