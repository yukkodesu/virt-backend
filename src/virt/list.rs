use roxmltree::Document;
use std::{collections::HashMap, sync::mpsc::Sender, vec};
use virt::{connect::Connect, domain::Domain, domain_snapshot::DomainSnapshot};

pub fn list_all(conn: &Connect, main_tx: &Sender<String>) {
    let t: Vec<HashMap<&str, String>> = conn
        .list_all_domains(0)
        .unwrap()
        .into_iter()
        .map(|dom| {
            let mut obj = HashMap::new();
            obj.insert("name", dom.get_name().unwrap());
            obj.insert("vcpu", dom.get_info().unwrap().nr_virt_cpu.to_string());
            obj.insert("memory", dom.get_info().unwrap().memory.to_string());
            obj
        })
        .collect();
    main_tx.send(serde_json::to_string(&t).unwrap()).unwrap();
}

pub fn list_snapshot(conn: &Connect, main_tx: &Sender<String>, params: &Vec<String>) {
    // create json_obj like
    // {
    //     "domain_name":[
    //         {
    //             "name": "snapshot_name",
    //             "description": "",
    //             "state": "",
    //             "creationTime": "",
    //         }
    //     ]
    // }
    let t: HashMap<&String, Vec<HashMap<&str, String>>> = params
        .into_iter()
        .map(|dom_name| match Domain::lookup_by_name(conn, dom_name) {
            Err(e) => {
                let mut obj = HashMap::new();
                obj.insert("error", e.to_string());
                (dom_name, vec![obj])
            }
            Ok(dom) => {
                let snapshots = dom.list_all_snapshots(0).unwrap();
                let snapshots: Vec<HashMap<&str, String>> = snapshots
                    .iter()
                    .map(|it| {
                        let info_str = &it.get_xml_desc(0).unwrap();
                        let info = roxmltree::Document::parse(info_str).unwrap();
                        let get_text_by_tagname = |info: &Document, tag_name: &str| -> String {
                            match info.descendants().find(|it| it.has_tag_name(tag_name)) {
                                Some(it) => it.text().unwrap_or("").to_string(),
                                None => "".to_string(),
                            }
                        };
                        let name = get_text_by_tagname(&info, "name");
                        let description = get_text_by_tagname(&info, "description");
                        let state = get_text_by_tagname(&info, "state");
                        let creation_time = get_text_by_tagname(&info, "creationTime");
                        let mut obj = HashMap::new();
                        obj.insert("name", name);
                        obj.insert("description", description);
                        obj.insert("state", state);
                        obj.insert("creationTime", creation_time);
                        obj
                    })
                    .collect();
                (dom_name, snapshots)
            }
        })
        .collect();
    main_tx.send(serde_json::to_string(&t).unwrap()).unwrap();
}

pub fn snapshot_current(conn: &Connect, main_tx: &Sender<String>, params: &Vec<String>) {
    let t: HashMap<&String, String> = params
        .into_iter()
        .map(|dom_name| {
            if let Ok(dom) = Domain::lookup_by_name(conn, &dom_name) {
                match DomainSnapshot::current(dom, 0u32) {
                    Ok(snapshot) => return (dom_name, snapshot.get_name().unwrap()),
                    Err(e) => return (dom_name, "None".to_string()),
                }
            } else {
                (dom_name, "None".to_string())
            }
        })
        .collect();
    main_tx.send(serde_json::to_string(&t).unwrap()).unwrap();
}

pub fn list_snapshot_tree(conn: &Connect, main_tx: &Sender<String>, params: &Vec<String>) {
    let dom_name = &params[0];
    let t = match Domain::lookup_by_name(conn, dom_name) {
        Ok(dom) => {
            let snapshots = dom.list_all_snapshots(0).unwrap();
            let snapshots: HashMap<String, Vec<String>> = snapshots
                .iter()
                .map(|it| {
                    let childs: Vec<String> = it
                        .list_all_children(0)
                        .unwrap()
                        .iter()
                        .map(|child| child.get_name().unwrap())
                        .collect();
                    (it.get_name().unwrap(), childs)
                })
                .collect();
            snapshots
        }
        Err(_) => HashMap::new(),
    };
    main_tx.send(serde_json::to_string(&t).unwrap()).unwrap();
}
