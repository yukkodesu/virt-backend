use serde::{Deserialize, Serialize};
use std::{
    sync::{
        mpsc::{self, Receiver, Sender, SyncSender},
        Mutex,
    },
    thread,
};
use sysinfo::System;
use thiserror::Error;
use virt::connect::Connect;

use self::conn::*;
use self::sys::*;

mod conn;
pub mod shell;
mod sys;
mod utils;

pub struct VirtCommand {
    cmd: VirtCommandType,
    params: Vec<String>,
}

#[derive(Error, Debug)]
pub enum VirtError {
    #[error("Domain {0} not found")]
    DomainNotFound(String),
    #[error("No Snapshot named {dom_name:?} in Domain {snapshot_name:?}")]
    SnapShotNotFound {
        dom_name: String,
        snapshot_name: String,
    },
    #[error("Input Invalid")]
    InvalidInput,
    #[error("System Internal Error")]
    VirtInternalError(#[from] virt::error::Error),
}

pub type VirtResult = Result<String, VirtError>;

pub enum VirtCommandType {
    ListAll,
    ListSnapshot,
    ListSnapshotTree,
    SysInfo,
    EditSnapshot,
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
    pub rx: Mutex<Receiver<VirtResult>>,
}

impl VirtConnect {
    pub fn new() -> Self {
        let (virt_tx, virt_rx): (SyncSender<VirtCommand>, Receiver<VirtCommand>) =
            mpsc::sync_channel(2);
        let (main_tx, main_rx): (Sender<VirtResult>, Receiver<VirtResult>) = mpsc::channel();
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
                        VirtCommandType::EditSnapshot => edit_snapshot(&conn, &main_tx, &params),
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

#[derive(Serialize, Deserialize)]
pub struct SnapShotConfig {
    pub dom_name: String,
    pub snapshot_name: String,
    pub description: Option<String>,
    pub is_live: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapShotEditConfig {
    pub dom_name: String,
    pub snapshot_name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AltDomStateCommand {
    pub dom_name: String,
    pub state: String,
}

#[derive(Deserialize, Serialize)]
enum SystemType {
    Linux,
    Windows,
}

#[derive(Deserialize, Serialize)]
pub struct CreateVirtConfig {
    virt_name: String,
    memory: String,
    vcpu: String,
    system: SystemType,
    iso_filename: String,
    disk_size: String,
}
