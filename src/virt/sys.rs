use std::{collections::HashMap, sync::mpsc::Sender, time::{SystemTime, UNIX_EPOCH}};
use sysinfo::System;
pub fn get_sysinfo(main_tx: &Sender<String>, sys: &mut System) {
    sys.refresh_cpu();
    sys.refresh_memory();

    let mut t = HashMap::new();

    let now = SystemTime::now();
    let unix_timestamp = now.duration_since(UNIX_EPOCH).unwrap().as_millis();
    t.insert("timestamp", unix_timestamp.to_string());
    t.insert("total memory", sys.total_memory().to_string());
    t.insert("used memory", sys.used_memory().to_string());
    t.insert("total swap", sys.total_swap().to_string());
    t.insert("used swap", sys.used_swap().to_string());
    t.insert("cpu number", sys.cpus().len().to_string());
    let mut cpu_usage = HashMap::new();
    let cpus = sys.cpus();
    for i in 0..sys.cpus().len() {
        cpu_usage.insert(i, cpus[i].cpu_usage());
    }
    t.insert("cpu usage", serde_json::to_string(&cpu_usage).unwrap());
    main_tx.send(serde_json::to_string(&t).unwrap()).unwrap();
}
