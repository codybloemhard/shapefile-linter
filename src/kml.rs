use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use xml::reader::{EventReader,XmlEvent};

fn indent(size: usize) -> String{
    const INDENT: &'static str = "  ";
    (0..size).map(|_| INDENT).fold(String::with_capacity(size*INDENT.len()), |r,s| r + s)
}

fn clean_name(name: String) -> String{
    let mut builder = String::new();
    let mut erase = false;
    for c in name.chars(){
        if c == '{'{
            erase = true;
            continue;
        }
        if c == '}'{
            erase = false;
            continue;
        }
        if !erase{
            builder.push(c);
        }
    }
    builder
}

pub fn print_xml_tag_tree(path: String){
    let file = File::open(&path);
    let file = if let Ok(ffile) = file{
        ffile
    }else{
        panic!("Could not open xml file");
    };
    let parser = EventReader::new(file);
    let mut depth = 0;
    for e in parser{
        match e{
            Ok(XmlEvent::StartElement { name, .. }) => {
                println!("{}-{}", indent(depth), clean_name(name.to_string()));
                depth += 1;
            }
            Ok(XmlEvent::EndElement { .. }) => {
                depth -= 1;
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
    println!("hoi");
}

pub fn print_xml_tag_count(path: String){
    let file = File::open(&path);
    let file = if let Ok(ffile) = file{
        ffile
    }else{
        panic!("Could not open xml file!");
    };
    let mut map = HashMap::new();
    let parser = EventReader::new(file);
    for e in parser{
        match e{
            Ok(XmlEvent::StartElement { name, .. }) => {
                let nname = clean_name(name.to_string());
                let newcount = match map.get(&nname){
                    Some(n) => { n + 1 },
                    None => { 1 },
                };
                map.insert(nname, newcount);
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
    let mut vec = Vec::new();
    for item in &map{
        vec.push(item);
    }
    vec.sort_by_key(|x| x.1);
    for (key,count) in vec{
        println!("Tag: {}, count: {}", key, count);
    }
}
