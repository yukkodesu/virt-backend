use std::{
    sync::{
        mpsc::{self, Receiver, Sender, SyncSender},
        Mutex,
    },
    thread,
};
use sysinfo::System;
use virt::connect::Connect;

use self::list::*;
use self::sys::*;

mod list;
pub mod shell;
mod sys;

pub struct VirtCommand {
    cmd: VirtCommandType,
    params: Vec<String>,
}

pub enum VirtCommandType {
    ListAll,
    ListSnapshot,
    ListSnapshotTree,
    SysInfo,
}

impl VirtCommand {
    pub fn create(cmd: VirtCommandType) -> Self {
        VirtCommand {
            cmd,
            params: Vec::new(),
        }
    }
    pub fn create_with_params(cmd: VirtCommandType, params: Vec<String>) -> Self {
        VirtCommand { cmd, params }
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
            let mut sys = System::new();
            loop {
                if let Ok(VirtCommand { cmd, params }) = virt_rx.recv() {
                    match cmd {
                        VirtCommandType::ListAll => list_all(&conn, &main_tx),
                        VirtCommandType::ListSnapshot => list_snapshot(&conn, &main_tx, &params),
                        VirtCommandType::ListSnapshotTree => {
                            list_snapshot_tree(&conn, &main_tx, &params)
                        }
                        VirtCommandType::SysInfo => get_sysinfo(&main_tx, &mut sys),
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
