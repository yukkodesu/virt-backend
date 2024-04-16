use crate::{
    db::entity::{prelude::*, *},
    middleware::authenticate::JWT,
};
use futures::{SinkExt, StreamExt};
use rocket::{http::Status, response::content, serde::json::Json, State};
use rocket_ws::{Channel, WebSocket};

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[get("/ws/<port>")]
pub async fn vnc_connect(_jwt: JWT, port: &str, ws: WebSocket) -> Channel<'_> {
    let mut socket_stream = TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .unwrap();
    let mut buffer: Vec<u8> = vec![0; 4096];
    println!("{}", port);
    ws.channel(move |mut ws_stream| {
        Box::pin(async move {
            loop {
                tokio::select! {
                    Some(message) = ws_stream.next() => {
                        if let Ok(message) = message {
                            let binary: Vec<u8>= message.into();
                            socket_stream.write(&binary).await.unwrap();
                        }
                        else {
                            break;
                        }
                    },
                    data_bytes = socket_stream.read(&mut buffer) => {
                        let data_bytes = data_bytes.unwrap();
                        if data_bytes > 0 {
                            let _ = ws_stream.send(buffer[..data_bytes].into()).await.unwrap();
                        }
                        else {
                            break;
                        }
                    }
                }
            }
            Ok(())
        })
    })
}

#[derive(Serialize, Deserialize)]
struct VncDisplayConfig {
    port: String,
    password: String,
}

#[post("/get-vnc-display-config", format = "application/json", data = "<dom_name>")]
pub async fn get_vnc_display_config(
    _jwt: JWT,
    db: &State<DatabaseConnection>,
    dom_name: Json<String>,
) -> (Status, content::RawJson<String>) {
    let db = db as &DatabaseConnection;
    let domain = match Domains::find()
        .filter(domains::Column::Name.eq(&dom_name.0))
        .one(db)
        .await
    {
        Ok(Some(v)) => v,
        _ => {
            return (
                Status::InternalServerError,
                content::RawJson(String::from(format!(
                    "Error: domain {} not found",
                    &dom_name.0
                ))),
            )
        }
    };
    let vnc_display_config = serde_json::to_string(&VncDisplayConfig {
        port: domain.vnc_port,
        password: domain.vnc_password,
    })
    .unwrap();
    (Status::Ok, content::RawJson(vnc_display_config))
}
