use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use std::collections::HashSet;
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
    builder.to_ascii_lowercase()
}

macro_rules! open_file{
    ($path:expr) => {
        if let Ok(ffile) = File::open(&$path){
            ffile
        }else{
            panic!("Could not open file: {}", $path);
        };
    };
}

pub fn print_xml_tag_tree(path: String){
    let file = open_file!(path);
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
    let file = open_file!(path);
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

pub fn kml_height(path: String){
    let file = open_file!(path);
    let parser = EventReader::new(file);
    let coord_name = String::from("coordinates");
    let mut coor = false;
    let mut strings = Vec::new();
    for e in parser{
        match e{
            Ok(XmlEvent::StartElement { name, .. }) => {
                let nname = clean_name(name.to_string());
                if nname != coord_name{ continue; }
                coor = true;
            }
            Ok(XmlEvent::Characters(content)) => {
                if !coor { continue; }
                strings.push(content);
            }
            Ok(XmlEvent::EndElement{ name }) => {
                let nname = clean_name(name.to_string());
                if nname != coord_name { continue; }
                coor = false;
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
    println!("{}", strings.len());
    let mut vvp4 = Vec::new();
    for string in strings{
        let mut line = Vec::new();
        let points_str: Vec<_> = string.split(' ').collect();
        for point_str in points_str{
            let comps: Vec<_> = point_str.split(',').collect();
            if comps.len() != 3 { continue; }
            let x = comps[0].parse::<f64>();
            let y = comps[1].parse::<f64>();
            let z = comps[2].parse::<f64>();
            if x.is_err() || y.is_err() || z.is_err(){
                panic!("xyz none");
            }
            fn cclamp<T: std::fmt::Debug>(c: Result<f64,T>) -> f64{
                (c.unwrap() / 5.0).round() * 5.0
            }
            line.push((
                    cclamp(x),
                    cclamp(y),
                    cclamp(z),0));
        }
        vvp4.push(line);
    }
    println!("{}", vvp4.len());
}
