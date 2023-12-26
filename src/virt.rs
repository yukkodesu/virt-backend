use std::{
    sync::{
        mpsc::{self, Receiver, Sender, SyncSender},
        Mutex,
    },
    thread,
};
use virt::connect::Connect;

use self::list::*;

mod list;

pub struct VirtCommand {
    cmd: String,
    params: Vec<String>,
}

impl VirtCommand {
    pub fn create(cmd: &str) -> Self {
        VirtCommand {
            cmd: cmd.to_string(),
            params: Vec::new(),
        }
    }
    pub fn create_with_params(cmd: &str, params: Vec<String>) -> Self {
        VirtCommand {
            cmd: cmd.to_string(),
            params,
        }
    }
}

pub struct VirtConnect {
    pub tx: SyncSender<VirtCommand>,
    pub rx: Mutex<Receiver<String>>,
}

impl VirtConnect {
    pub fn new() -> Self {
        let (virt_tx, virt_rx): (SyncSender<VirtCommand>, Receiver<VirtCommand>) =
            mpsc::sync_channel(2);
        let (main_tx, main_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        thread::spawn(move || {
            let mut conn = Connect::open("qemu:///session").expect("connection err");

            loop {
                if let Ok(VirtCommand { cmd, params }) = virt_rx.recv() {
                    match cmd.as_str() {
                        "ListAll" => list_all(&conn, &main_tx),
                        "ListSnapshot" => list_snapshot(&conn, &main_tx, &params),
                        _ => (),
                    }
                } else {
                    conn.close().unwrap();
                    break;
                }
            }
        });
        VirtConnect {
            tx: virt_tx,
            rx: Mutex::new(main_rx),
        }
    }
}
