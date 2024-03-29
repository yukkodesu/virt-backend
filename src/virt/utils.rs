use std::io::Cursor;

use data_encoding::HEXUPPER;
use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Reader, Writer,
};
use ring::digest::{Context, SHA256};
pub fn edit_xml_text(
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

pub fn sha256_hash(str: &str) -> String {
    let mut context = Context::new(&SHA256);
    context.update(str.as_bytes());
    HEXUPPER.encode(context.finish().as_ref())
}
