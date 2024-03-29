use rocket::{http::{Cookie, Status}, local::asynchronous::Client};
use serde::{Deserialize, Serialize};

use super::get_auth;
use crate::build;

#[rocket::async_test]
async fn list_all() {
    let client = Client::tracked(build().await).await.unwrap();
    let auth = Cookie::new("authorization", get_auth(&client).await);
    let response = client
        .get(uri!("/api/list-all"))
        .cookie(auth.clone())
        .dispatch()
        .await;
    let result = r#"[
        {
            "name": "win11",
            "vcpu": "2",
            "memory": "8388608"
        },
        {
            "name": "debian",
            "vcpu": "2",
            "memory": "8388608"
        }
    ]"#;
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Domain {
        name: String,
        vcpu: String,
        memory: String,
    }
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        serde_json::from_str::<Vec<Domain>>(&response.into_string().await.unwrap()).unwrap(),
        serde_json::from_str::<Vec<Domain>>(result).unwrap()
    );
}
