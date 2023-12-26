use std::{collections::HashMap, sync::mpsc::Sender};
use virt::{connect::Connect, domain::Domain, domain_snapshot::DomainSnapshot};

pub fn list_all(conn: &Connect, main_tx: &Sender<String>) {
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

pub fn list_snapshot(conn: &Connect, main_tx: &Sender<String>, params: &Vec<String>) {
    let t: Vec<HashMap<&str, String>> = params
        .into_iter()
        .map(|dom_name| {
            let mut map = HashMap::new();
            match Domain::lookup_by_name(conn, dom_name) {
                Ok(dom) => {
                    let snapshots = dom.list_all_snapshots(0).unwrap();
                    for snapshot in &snapshots {
                        let info_str = snapshot.get_xml_desc(0).unwrap();
                        map.insert(dom_name.as_str(), info_str);
                    }
                }
                Err(e) => (),
            }
            map
        })
        .filter(|map| !map.is_empty())
        .collect();
    main_tx.send(serde_json::to_string(&t).unwrap()).unwrap();
}
