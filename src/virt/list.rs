use roxmltree::Document;
use std::{collections::HashMap, sync::mpsc::Sender, vec};
use virt::{connect::Connect, domain::Domain};

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
    let t: HashMap<&String, Vec<HashMap<&str, String>>> = params
        .into_iter()
        .map(|dom_name| {
            // let mut map = HashMap::new();
            match Domain::lookup_by_name(conn, dom_name) {
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
                                info.descendants()
                                    .find(|it| it.has_tag_name(tag_name))
                                    .unwrap()
                                    .text()
                                    .unwrap()
                                    .to_string()
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
            }
        })
        .collect();
    main_tx.send(serde_json::to_string(&t).unwrap()).unwrap();
}
