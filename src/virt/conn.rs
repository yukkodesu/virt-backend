use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use roxmltree::Document;
use std::io::Cursor;
use std::{collections::HashMap, sync::mpsc::Sender};
use virt::{connect::Connect, domain::Domain, domain_snapshot::DomainSnapshot};

use super::VirtError::{self, *};
use super::{SnapShotEditConfig, VirtResult};

pub fn list_all(conn: &Connect, main_tx: &Sender<VirtResult>) {
    match conn.list_all_domains(0) {
        Ok(doms) => {
            let t: Vec<HashMap<&str, String>> = doms
                .into_iter()
                .map(|dom| {
                    let mut obj = HashMap::new();
                    obj.insert("name", dom.get_name().expect("Domain must have name!"));
                    let dom_info = dom.get_info().expect("Domain must have Info!");
                    obj.insert("vcpu", dom_info.nr_virt_cpu.to_string());
                    obj.insert("memory", dom_info.memory.to_string());
                    obj
                })
                .collect();
            main_tx
                .send(VirtResult::Ok(serde_json::to_string(&t).unwrap()))
                .unwrap();
        }
        Err(e) => main_tx.send(VirtResult::Err(VirtInternalError(e))).unwrap(),
    }
}

pub fn list_snapshot(conn: &Connect, main_tx: &Sender<VirtResult>, params: &Vec<String>) {
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

    let get_text_by_tagname = |info: &Document, tag_name: &str| -> String {
        match info.descendants().find(|it| it.has_tag_name(tag_name)) {
            Some(it) => it.text().unwrap_or("").to_string(),
            None => "".to_string(),
        }
    };

    let mut t: HashMap<&String, Vec<HashMap<&str, String>>> = HashMap::new();

    let res = params
        .into_iter()
        .try_for_each(|dom_name| -> Result<(), VirtError> {
            match Domain::lookup_by_name(conn, dom_name) {
                Err(e) => Err(DomainNotFound(dom_name.clone())),
                Ok(dom) => match dom.list_all_snapshots(0) {
                    Err(e) => Err(VirtInternalError(e)),
                    Ok(snapshots) => {
                        let snapshots: Vec<HashMap<&str, String>> = snapshots
                            .iter()
                            .map(|it| {
                                let info_str = &it.get_xml_desc(0).unwrap();
                                let info = roxmltree::Document::parse(info_str)
                                    .expect("XML from LibVirt can't be parsed");
                                let name = get_text_by_tagname(&info, "name");
                                let description = get_text_by_tagname(&info, "description");
                                let state = get_text_by_tagname(&info, "state");
                                let creation_time = get_text_by_tagname(&info, "creationTime");
                                let is_current = it.is_current(0u32).unwrap();
                                let mut obj = HashMap::new();
                                obj.insert("name", name);
                                obj.insert("description", description);
                                obj.insert("state", state);
                                obj.insert("creationTime", creation_time);
                                obj.insert("isCurrent", is_current.to_string());
                                obj
                            })
                            .collect();
                        t.insert(dom_name, snapshots);
                        Ok(())
                    }
                },
            }
        });
    match res {
        Ok(_) => main_tx
            .send(VirtResult::Ok(serde_json::to_string(&t).unwrap()))
            .unwrap(),
        Err(e) => main_tx.send(VirtResult::Err(e)).unwrap(),
    };
}

pub fn list_snapshot_tree(conn: &Connect, main_tx: &Sender<VirtResult>, params: &Vec<String>) {
    let dom_name = &params[0];
    match Domain::lookup_by_name(conn, dom_name) {
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
            main_tx
                .send(VirtResult::Ok(serde_json::to_string(&snapshots).unwrap()))
                .unwrap();
        }
        Err(e) => main_tx
            .send(VirtResult::Err(VirtError::DomainNotFound(dom_name.clone())))
            .unwrap(),
    }
}

fn edit_xml_text(
    input: &String,
    target_element: &str,
    new_text: &str,
    defined_depth: i64,
) -> String {
    let mut reader = Reader::from_str(&input);
    reader.trim_text(true);
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
    let mut in_target_element = false;
    let mut is_exist = false;
    let mut depth = 0 as i64;
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                depth += 1;
                if e.name().as_ref() == target_element.as_bytes() {
                    in_target_element = true;
                    is_exist = true;
                }
                writer.write_event(Event::Start(e.to_owned())).unwrap();
            }
            Ok(Event::End(e)) => {
                if depth == defined_depth && !is_exist {
                    let elem_start = BytesStart::new(target_element);
                    writer.write_event(Event::Start(elem_start)).unwrap();
                    let text = BytesText::new(new_text);
                    writer.write_event(Event::Text(text)).unwrap();
                    let elem_end = BytesEnd::new(target_element);
                    writer.write_event(Event::End(elem_end)).unwrap();
                }
                depth -= 1;
                if e.name().as_ref() == target_element.as_bytes() {
                    in_target_element = false;
                }
                writer.write_event(Event::End(e.to_owned())).unwrap();
            }
            Ok(Event::Text(e)) => {
                if in_target_element && depth == defined_depth + 1 {
                    writer
                        .write_event(Event::Text(BytesText::new(new_text)))
                        .unwrap();
                } else {
                    writer.write_event(Event::Text(e.to_owned())).unwrap();
                }
            }
            Ok(Event::Eof) => break,
            Ok(e) => writer.write_event(e).unwrap(),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).unwrap()
}

pub fn edit_snapshot(conn: &Connect, main_tx: &Sender<VirtResult>, params: &Vec<String>) {
    match serde_json::from_str::<SnapShotEditConfig>(&params[0]) {
        Ok(config) => {
            println!("{:?}", config);
            match Domain::lookup_by_name(conn, &config.dom_name) {
                Ok(dom) => match DomainSnapshot::lookup_by_name(&dom, &config.snapshot_name, 0) {
                    Ok(snapshot) => {
                        let xml_str = snapshot.get_xml_desc(0).unwrap();
                        let mut new_xml =
                            edit_xml_text(&xml_str, "name", &config.new_snapshot_name, 1);
                        if let Some(description) = config.description {
                            new_xml =
                                edit_xml_text(&new_xml, "description", &description, 1);
                        }
                        // println!("{}", new_xml);
                        main_tx
                            .send(VirtResult::Ok("Successfully edit snapshot".to_string()))
                            .unwrap();
                    }
                    Err(_) => main_tx
                        .send(VirtResult::Err(VirtError::SnapShotNotFound {
                            dom_name: config.dom_name.to_string(),
                            snapshot_name: config.snapshot_name.to_string(),
                        }))
                        .unwrap(),
                },
                Err(_) => main_tx
                    .send(VirtResult::Err(VirtError::DomainNotFound(
                        config.dom_name.to_string(),
                    )))
                    .unwrap(),
            }
        }
        Err(_) => main_tx
            .send(VirtResult::Err(VirtError::InvalidInput))
            .unwrap(),
    }
}
