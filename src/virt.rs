use std::{
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender, SyncSender},
        Mutex,
    },
    thread,
};
use virt::connect::Connect;

pub struct VirtConnect {
    pub tx: SyncSender<String>,
    pub rx: Mutex<Receiver<String>>,
}

impl VirtConnect {
    pub fn new() -> Self {
        let (virt_tx, virt_rx): (SyncSender<String>, Receiver<String>) = mpsc::sync_channel(2);
        let (main_tx, main_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        thread::spawn(move || {
            let mut conn = Connect::open("qemu:///session").expect("connection err");

            loop {
                if let Ok(val) = virt_rx.recv() {
                    println!("{}", val);
                    match val.as_str() {
                        "ListAll" => list_all(&conn, &main_tx),
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

fn list_all(conn: &Connect, main_tx: &Sender<String>) {
    let t: Vec<HashMap<&str, String>> = conn
        .list_all_domains(0)
        .unwrap()
        .into_iter()
        .map(|dom| {
            let mut map = HashMap::new();
            map.insert("name", dom.get_name().unwrap());
            map.insert("vcpu", dom.get_info().unwrap().nr_virt_cpu.to_string());
            map.insert("memory", dom.get_info().unwrap().memory.to_string());
            map
        })
        .collect();
    main_tx.send(serde_json::to_string(&t).unwrap()).unwrap();
}
