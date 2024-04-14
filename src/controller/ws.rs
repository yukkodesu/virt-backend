use crate::middleware::authenticate::JWT;
use futures::{SinkExt, StreamExt};
use rocket_ws::{Channel, Message, WebSocket};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[get("/ws/<port>")]
pub async fn vnc_connect(port: &str, ws: WebSocket) -> Channel<'_> {
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
