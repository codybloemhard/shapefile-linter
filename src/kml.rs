use std::fs::File;
use std::collections::{HashMap,HashSet};
use xml::reader::{EventReader,XmlEvent};
use crate::data::{VvP4,P4};
use crate::convert::degree_to_utm;
use crate::logger::*;
use hex::FromHex;
use std::str::FromStr;
// right amount of spaces for x indentations
fn indent(size: usize) -> String{
    const INDENT: &'static str = "  ";
    (0..size).map(|_| INDENT).fold(String::with_capacity(size*INDENT.len()), |r,s| r + s)
}
// clean out everything between {} inclusive, and to lowercase
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
// open file or die
macro_rules! open_file{
    ($path:expr) => {
        if let Ok(ffile) = File::open($path){
            ffile
        }else{
            panic!("Could not open file: {}", $path);
        };
    };
}
// print a indented tree of opening xml tags
pub fn print_xml_tag_tree(path: &str){
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
}
// check if a certain tags is always present in another tag
pub fn check_tag_child(path: &str, parent: &str, child: &str) -> bool{
    let file = open_file!(path);
    let parser = EventReader::new(file);
    let mut inside = false;
    let mut seen = false;
    for e in parser{
        match e{
            Ok(XmlEvent::StartElement { name, .. }) => {
                let cleaned = clean_name(name.to_string());
                if &cleaned == parent{ inside = true; seen = false; }
                else if &cleaned == child && inside { seen = true; }
            }
            Ok(XmlEvent::EndElement { name, .. }) => {
                let cleaned = clean_name(name.to_string());
                if &cleaned == parent{
                    if !inside { panic!("Found end tag of parent with seeing the start tag, somehow..."); }
                    if !seen{
                        return false;
                    }
                    inside = false;
                    seen = false;
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
    true
}
pub fn check_nonempty_tag(path: &str, tag: &str) -> bool{
    let file = open_file!(path);
    let parser = EventReader::new(file);
    let mut in_tag = false;
    let mut inside = String::new();
    for e in parser{
        match e{
            Ok(XmlEvent::StartElement { name, .. }) => {
                let nname = clean_name(name.to_string());
                if &nname == tag{
                    in_tag = true;
                }
            }
            Ok(XmlEvent::Characters(content)) => {
                if in_tag{
                    inside = content;
                }
            }
            Ok(XmlEvent::EndElement{ name }) => {
                let nname = clean_name(name.to_string());
                if &nname == tag{
                    if &inside == "" { return false; }
                    in_tag = false;
                    inside = String::new();
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
    true
}
// count all tags and print them out with counts
pub fn print_xml_tag_count(path: &str){
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
// parse string of coordinates
pub fn parse_coords(string: String) -> Vec<P4<f64>>{
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
        fn hclamp<T>(c: Result<f64,T>) -> f64{
            (c.unwrap_or_default() / 5.0).round() * 5.0
        }
        let (_,_,x,y) = degree_to_utm((x.unwrap(),y.unwrap()));
        line.push((x, y, hclamp(z), 0.0));
    }
    line
}
// parse heightlines from kml file
pub fn kml_height(path: &str) -> VvP4{
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
        vvp4.push(parse_coords(string));
    }
    vvp4
}
//parse polygons from geological kml file
pub fn kml_geo(path: &str, styles: &mut Vec<(u8,u8,u8,u8)>, counter: &mut usize, logger: &mut Logger) -> Vec<(usize,(VvP4,VvP4))>{
    let file = open_file!(path);
    let parser = EventReader::new(file);
    let mut colset = HashSet::new();
    let mut colmap = HashMap::new();
    let mut in_poly_style = false;
    let mut style_id = String::new();
    let mut in_colour = false;
    let mut in_outline = false;
    let mut colour = String::new();
    let mut outline = '0';
    let mut styles_raw = Vec::new();
    let mut in_style_url = false;
    let mut style_url = String::new();
    let mut in_outer = false;
    let mut in_inner = false;
    let mut in_coordinates = false;
    let mut outers = Vec::new();
    let mut inners = Vec::new();
    let mut polygons = Vec::new();
    for e in parser{
        match e{
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                let nname = clean_name(name.to_string());
                if &nname == "style"{
                    if attributes.len() != 1{
                        panic!("style tag should have only one attribute: id");
                    }
                    style_id = attributes[0].value.clone();
                }else if &nname == "polystyle"{
                    in_poly_style = true;
                }else if &nname == "color" && in_poly_style{// fool, orang is coloure!
                    in_colour = true;
                }else if &nname == "outline" && in_poly_style{
                    in_outline = true;
                }else if &nname == "styleurl"{
                    in_style_url = true;
                }else if &nname == "outerboundaryis"{
                    in_outer = true;
                }else if &nname == "innerboundaryis"{
                    in_inner = true;
                }else if &nname == "coordinates"{
                    in_coordinates = true;
                }
            }
            Ok(XmlEvent::Characters(content)) => {
                if in_colour{
                    colour = content;
                }else if in_outline{
                    if content.is_empty() { panic!("Outline tag content can not be empty!"); }
                    outline = content.chars().next().unwrap_or('0');
                }else if in_style_url{
                    style_url = content;
                }else if in_coordinates && in_outer{
                    outers.push(content);
                }else if in_coordinates && in_inner{
                    inners.push(content);
                }
            }
            Ok(XmlEvent::EndElement{ name }) => {
                let nname = clean_name(name.to_string());
                if &nname == "polystyle" {
                    in_poly_style = false;
                    styles_raw.push((style_id,colour,outline));
                    style_id = String::new(); // befriend the borrowchecker
                    colour = String::new(); // by giving him crap to eat
                }
                else if &nname == "color" { in_colour = false; }
                else if &nname == "outline" { in_outline = false; }
                else if &nname == "styleurl" { in_style_url = false; }
                else if &nname == "outerboundaryis" { in_outer = false; }
                else if &nname == "innerboundaryis" { in_inner = false; }
                else if &nname == "coordinates" { in_coordinates = false; }
                else if &nname == "polygon" {
                    polygons.push((style_url.clone(),outers,inners));
                    outers = Vec::new();
                    inners = Vec::new();
                }else if &nname == "placemark" {
                    style_url = String::new();
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
    for (id,colourstr,outline) in styles_raw{
        if colset.contains(&id) { continue; }
        colmap.insert(id.clone(), *counter);
        colset.insert(id.clone());
        let outl = if outline == '1' { 1u8 }
        else { 0u8 };
        let components = if let Ok(c) = Vec::from_hex(colourstr)
        { c } else { panic!("Could not parse hex colour!"); };
        if components.len() < 3 {
            panic!("Expected at least 3 components in colour!");
        }
        let offset = components.len() - 3;
        let r = components[offset];
        let g = components[offset + 1];
        let b = components[offset + 2];
        styles.push((outl,r,g,b));
        *counter += 1;
    }
    let mut polys = Vec::new();
    for (sturl,outersraw,innersraw) in polygons{
        if &sturl == ""{
            logger.log(Issue::EmptyStyleId);
            continue;
        }
        let id = if let Some(idd) = colmap.get(&sturl.chars().filter(|c| *c != '#').collect::<String>())
        { *idd } else {
            logger.log(Issue::MissingStyleId);
            continue;
        };
        let mut outers = Vec::new();
        for outerraw in outersraw{
            outers.push(parse_coords(outerraw));
        }
        let mut inners = Vec::new();
        for innerraw in innersraw{
            inners.push(parse_coords(innerraw));
        }
        polys.push((id,(outers,inners)));
    }
    polys
}

//parse lines from geological kml file
pub fn kml_geo_lines(path: &str, styles: &mut Vec<(u8,u8,u8,u8)>, counter: &mut usize, logger: &mut Logger) -> Vec<(usize,VvP4)>{
    let file = open_file!(path);
    let parser = EventReader::new(file);
    let mut colset = HashSet::new();
    let mut colmap = HashMap::new();
    let mut in_line_style = false;
    let mut style_id = String::new();
    let mut in_colour = false;
    let mut in_width = false;
    let mut colour = String::new();
    let mut width = String::new();
    let mut styles_raw = Vec::new();
    let mut in_style_url = false;
    let mut style_url = String::new();
    let mut in_outer = false;
    let mut in_inner = false;
    let mut in_coordinates = false;
    let mut in_outline = false;
    let mut outline = '-';
    let mut in_line = false;
    let mut lines = Vec::new();
    let mut polygons = Vec::new();
    for e in parser{
        match e{
            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                let nname = clean_name(name.to_string());
                if &nname == "style"{
                    if attributes.len() != 1{
                        panic!("style tag should have only one attribute: id");
                    }
                    style_id = attributes[0].value.clone();
                }else if &nname == "linestyle"{
                    in_line_style = true;
                }else if &nname == "color" && in_line_style{// fool, orang is coloure!
                    in_colour = true;
                }else if &nname == "width" && in_line_style{
                    in_width = true;
                }else if &nname == "styleurl"{
                    in_style_url = true;
                }else if &nname == "outerboundaryis"{
                    in_outer = true;
                }else if &nname == "innerboundaryis"{
                    in_inner = true;
                }else if &nname == "linestring"{
                    in_line = true;
                }else if &nname == "coordinates"{
                    in_coordinates = true;
                }else if &nname == "outline"{
                    in_outline = true;
                }
            }
            Ok(XmlEvent::Characters(content)) => {
                if in_colour{
                    colour = content;
                }else if in_width{
                    if content.is_empty() { panic!("Width should have content!"); }
                    width = content;
                }else if in_style_url{
                    style_url = content;
                }else if in_coordinates && (in_outer || in_inner){
                    lines.push(content);
                }else if in_coordinates && in_line{
                    lines.push(content);
                }else if in_outline{
                    outline = content.chars().next().expect("Outline content should have at least on char!");
                }
            }
            Ok(XmlEvent::EndElement{ name }) => {
                let nname = clean_name(name.to_string());
                if &nname == "style" {
                    styles_raw.push((style_id,colour,width,outline));
                    style_id = String::new(); // befriend the borrowchecker
                    colour = String::new(); // by giving him crap to eat
                    width = String::new(); // take this mr crab
                    outline = '-';
                }
                else if &nname == "color" { in_colour = false; }
                else if &nname == "width" { in_width = false; }
                else if &nname == "styleurl" { in_style_url = false; }
                else if &nname == "outerboundaryis" { in_outer = false; }
                else if &nname == "innerboundaryis" { in_inner = false; }
                else if &nname == "coordinates" { in_coordinates = false; }
                else if &nname == "outline" { in_outline = false; }
                else if &nname == "linestyle" { in_line_style = false }
                else if &nname == "polygon" {
                    polygons.push((style_url.clone(),lines));
                    lines = Vec::new();
                }
                else if &nname == "linestring"{
                    polygons.push((style_url.clone(),lines));
                    lines = Vec::new();
                }
                else if &nname == "placemark" {
                    style_url = String::new();
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
            _ => {}
        }
    }
    let mut skipmap = HashSet::new();
    for (id,colourstr,width,outline) in styles_raw{
        if colset.contains(&id) { continue; }
        if outline == '0' {
            skipmap.insert(id.clone());
            continue;
        }
        colmap.insert(id.clone(), *counter);
        colset.insert(id.clone());
        let width_int = if let Ok(v) = f32::from_str(&width) { (v * 10.0) as u8 }
        else { panic!("Could not parse width as float!"); };
        let components = if let Ok(c) = Vec::from_hex(colourstr)
        { c } else { panic!("Could not parse hex colour!"); };
        if components.len() < 3 {
            panic!("Expected at least 3 components in colour!");
        }
        let offset = components.len() - 3;
        let r = components[offset];
        let g = components[offset + 1];
        let b = components[offset + 2];
        styles.push((width_int,r,g,b));
        *counter += 1;
    }
    let mut res = Vec::new();
    for (sturl,rawlines) in polygons{
        let url = &sturl.chars().filter(|c| *c != '#').collect::<String>();
        if skipmap.contains(url){ continue; }
        if url == ""{
            logger.log(Issue::EmptyStyleId);
            continue;
        }
        let id = if let Some(idd) = colmap.get(url)
        { *idd } else {
            logger.log(Issue::MissingStyleId);
            continue;
        };
        let mut lines = Vec::new();
        for linesraw in rawlines{
            lines.push(parse_coords(linesraw));
        }
        res.push((id,lines));
    }
    res
}
