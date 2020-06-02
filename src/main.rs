extern crate lapp;
extern crate shapefile;
extern crate bin_buffer;
extern crate dlv_list;
extern crate xml;
extern crate hex;
extern crate ass;

use bin_buffer::*;
use std::path::Path;
use std::time::Instant;
use std::collections::{HashMap};

pub mod data;
pub mod info;
pub mod compress;
pub mod logger;
pub mod chunkify;
pub mod triangulate;
pub mod kml;
pub mod convert;

use info::*;
use compress::*;
use logger::*;
use data::*;
use crate::data::{PolygonZ};
use chunkify::*;
use kml::*;

fn main(){
    do_things();
}

fn do_things() -> Option<()>{
    // Set up cli arguments
    let args = lapp::parse_args("
    Preprocess shapefiles into more efficient files.
      <inputfile> (string...) input file(s) name(s)
      --output (default outp) (string) define output file
      --mode (default info) (string) what to do
      --ft (default none) (string) type of input file
      --tag0 (default none) (string) xml tag variable
      --tag1 (default none) (string) xml tag variable
      --cuts (default 1) number of cuts when chunking
      --cuts_multi (default 2) subdivide multiplier
      --levels (default 6) how many LOD's we have
      --mods (integer...) heightline modulo's
      "
    );

    let infiles = args.get_strings("inputfile");
    let outfile = args.get_string("output");
    let mode = args.get_string("mode");
    let ft = args.get_string("ft");
    let tag0 = args.get_string("tag0");
    let tag1 = args.get_string("tag1");
    let cuts = args.get_integer("cuts");
    let cuts_multi = args.get_integer("cuts_multi");
    let levels = args.get_integer("levels");
    let mods = args.get_integers("mods");

    let mut logger = Logger::default();

    println!("Shapefile processor...");
    let timer = Instant::now();
    // Take one file
    let get_only_path = ||{
        if infiles.is_empty() {
            logger.report();
            return Option::None;
        }
        Option::Some(infiles[0].clone())
    };
    // Take all files
    let read_single_file = |infile: String|{
        if let Ok(shapes) = shapefile::read(&infile){
            println!("Read file \"{}\": {} ms, shapes: {}", infile, timer.elapsed().as_millis(), shapes.len());
            Option::Some(shapes)
        }else{
            println!("Could not read file: {}", infile);
            Option::None
        }
    };
    // Read one file, assume it's the only one, more can't hurt.
    let read_only_file = ||{
        if infiles.is_empty() {
            logger.report();
            return Option::None;
        }
        read_single_file(infiles[0].clone())
    };
    let write_buffer = |filename: &str, buffer: &Buffer, timer: &std::time::Instant|{
        let ok = buffer_write_file(&Path::new(filename), buffer);
        println!("Writing file \"{}\", went ok?: {}, {} ms", "styles", ok, timer.elapsed().as_millis());
    };
    // Compress and bufferize and write collection.
    macro_rules! compress_and_write{
        ($col:expr) =>{
            let infos = info_package(&$col);
            let buffer = $col.compress(infos, &mut logger);
            write_buffer(&outfile, &buffer, &timer);
        }
    }
    // read in heightlines. depending on the choice the user made, shapefile or kml.
    // shapefile is assumed to be in utm and kml is assumed to be in lat/lon.
    macro_rules! get_plinezs{
        ($path:expr) => {
            if &ft == "none"{
                println!("No filetype specified!");
                logger.report();
                return None;
            }else if &ft == "shape"{
                let shapes = read_single_file($path)?;
                split(shapes, &mut logger).5
            }else if &ft == "kml"{
                kml_height(&$path)
            }else{
                println!("Unknown filetype specified!");
                logger.report();
                return None;
            };
        }
    }
    if mode == "shapeinfo"{// Just print info about shapefile content.
        if &ft != "shape" {
            println!("This mode only works on shapefiles!");
            logger.report();
            return None;
        }
        let shapes = read_only_file()?;
        print_shape_content(&shapes);
        let splitted = split(shapes, &mut logger);
        print_split_content(&splitted);
    }else if mode == "mergeheight"{// Take many heightfiles and combine them into one big compressed one.
        println!("{:?}", infiles);
        let mut collection = Vec::new();
        for file in infiles{
            let plinezs = get_plinezs!(file);
            let mut shapezs = compress_heightmap(plinezs, &mut logger);
            collection.append(&mut shapezs);
        }
        compress_and_write!(collection);
    }else if mode == "lintheight"{// Print info about heightlines
        let mut wrongs = Vec::new();
        for file in infiles{
            let plinezs = get_plinezs!(file);
            let mut vec = collect_wrong_heightlines(plinezs, &mut logger);
            wrongs.append(&mut vec);
        }
        println!("There are {} wrong heightlines", wrongs.len());
        let mut diffs = Vec::new();
        let mut sames = Vec::new();
        let mut lens = Vec::new();
        for (i,wrong) in wrongs.iter().enumerate(){
            let min = wrong.iter().fold(std::f64::MAX, |m,x| m.min(*x));
            let max = wrong.iter().fold(std::f64::MIN, |m,x| m.max(*x));
            let mut countmap = HashMap::new();
            for x in wrong{
                let mut y = 0usize;
                unsafe{ y = std::mem::transmute(x); }
                let newcount = match countmap.get(&y){
                    Some(n) => { n + 1 },
                    None => { 1 },
                };
                countmap.insert(y, newcount);
            }
            let mut vec = Vec::new();
            for item in &countmap{
                vec.push(item);
            }
            vec.sort_by_key(|x| x.1);
            let same = if vec.is_empty(){ 0 }
            else { *vec[vec.len() - 1].1 };
            let diff = max - min;
            diffs.push(diff);
            sames.push(same);
            lens.push(wrong.len());
            println!("Line {}: min: {} max: {} diff: {} same: {} len: {}", i, min, max, diff, same, wrong.len());
        }
        if diffs.is_empty(){
            println!("median: 0\nmean: 0");
            logger.report();
            return Option::Some(());
        }
        let mean = diffs.iter().fold(0.0, |sum,x| sum + x) / diffs.len() as f64;
        let min = diffs.iter().fold(std::f64::MAX, |m,x| m.min(*x));
        let max = diffs.iter().fold(std::f64::MIN, |m,x| m.max(*x));
        let mut diffs: Vec<u64> = diffs.iter().map(|x| *x as u64).collect::<Vec<u64>>();
        diffs.sort();
        let median = diffs[diffs.len() / 2];
        lens.sort();
        sames.sort();
        let lens_median = lens[lens.len() / 2];
        let sames_median = sames[sames.len() / 2];
        println!("Differences between min and max in lines, summary:");
        println!("median: {}", median);
        println!("mean: {}", mean);
        println!("min: {}", min);
        println!("max: {}", max);
        println!("length median: {}", lens_median);
        println!("same value's median: {}", sames_median);
    }else if mode == "chunkify"{// Take one compressed height file and build chunks from it.
        let string_path = &get_only_path()?;
        let path = std::path::Path::new(string_path);
        let mut buffer = ReadBuffer::from_raw(buffer_read_file(&path)?);
        let mx = u64::from_buffer(&mut buffer)?;
        let my = u64::from_buffer(&mut buffer)?;
        let mz = u64::from_buffer(&mut buffer)?;
        let multi = u64::from_buffer(&mut buffer)?;
        let bmin = <(u16,u16,u16)>::from_buffer(&mut buffer)?;
        let bmax = <(u16,u16,u16)>::from_buffer(&mut buffer)?;
        let shapes = <std::vec::Vec<ShapeZ<u16>> as Bufferable>::from_buffer(&mut buffer)?;
        println!("mx: {} my: {} mz: {} multi: {}", mx, my, mz, multi);
        println!("{:?}{:?}", bmin, bmax);
        print_height_distribution(&shapes);
        let cuts_mul = if cuts_multi > 0 { levels as u64 }
        else { panic!("Cuts multiplier must be at least one!"); };
        let mut cuts = if cuts > 0 { cuts as u64 }
        else { panic!("Cuts should be at least one!"); };
        let mut info_buffer = Vec::new();
        let levels = if levels > 0 { levels as u64 }
        else { panic!("Levels can not be smaller than one!"); };
        levels.into_buffer(&mut info_buffer);
        let mods = if !mods.is_empty() { mods.into_iter().map(|x| x as u64).collect::<Vec<_>>() }
        else { vec![400,200,100,50,25,5] };
        if mods.len() != levels as usize{
            panic!("Mods lenght must equal levels!");
        }
        for i in 0..levels{
            for (x,y,chunk) in cut(cuts.max(1), (bmin,bmax), &shapes, &mut logger){
                let points0 = chunk.iter().fold(0, |sum, sz| sum + sz.points_len());
                let mut buffer = Vec::new();
                i.into_buffer(&mut buffer);
                x.into_buffer(&mut buffer);
                y.into_buffer(&mut buffer);
                let filtered = pick_heights(mods[i as usize], chunk);
                let points1 = filtered.iter().fold(0, |sum, sz| sum + sz.points_len());
                let max = if mods[i as usize] <= 5 { std::usize::MAX } else { 5000 };
                let picked = pick_points(max, filtered);
                let points2 = picked.iter().fold(0, |sum, sz| sum + sz.points_len());
                let lines0 = picked.len();
                let mut lines1 = lines0;
                let finalv = if i < 4 {
                    let opti = optimize_lines(picked);
                    lines1 = opti.len();
                    opti
                }else{
                    picked
                };
                finalv.into_buffer(&mut buffer);
                let filename = &format!("{}-{}-{}.hlinechunk", i, x, y);
                write_buffer(filename, &buffer, &timer);
                println!("l0: {} l1: {} l2: {} s0: {} s1: {}", points0, points1, points2, lines0, lines1);
            }
            cuts.into_buffer(&mut info_buffer);
            cuts *= cuts_mul;
        }
        mx.into_buffer(&mut info_buffer);
        my.into_buffer(&mut info_buffer);
        mz.into_buffer(&mut info_buffer);
        multi.into_buffer(&mut info_buffer);
        bmin.into_buffer(&mut info_buffer);
        bmax.into_buffer(&mut info_buffer);
        mods.into_buffer(&mut info_buffer);
        let ok = buffer_write_file(&Path::new("chunks.info"), &info_buffer);
        println!("Writing file \"chunks.info\" ok?: {}", ok);
    }else if mode == "polygonz"{// Take shapefile and compress the polygonZ's
        let shapes = read_only_file()?;
        let polys = split(shapes, &mut logger).11;
        let polyzs: Vec<PolygonZ<f64>> = polys.into_iter().map(|p| PolygonZ::from(p,0)).collect();
        let infos = info_package(&polyzs);
        let buffer = polyzs.compress(infos, &mut logger);
        println!("Bufferized: {} ms", timer.elapsed().as_millis());
        write_buffer(&outfile, &buffer, &timer);
    }else if mode == "triangulate"{ // take polygonz's and triangulate and compress them
        let shapes = read_only_file()?;
        let polys = split(shapes, &mut logger).11;
        let polyzs: Vec<PolygonZ<f64>> = polys.into_iter().map(|p| PolygonZ::from(p,0)).collect();
        let infos = info_package(&polyzs);
        let buffer = polyzs.triangle_compress(infos, &mut logger);
        write_buffer(&outfile, &buffer, &timer);
    }else if mode == "height"{// Compress shapefile, assuming it consist of height lines.
        let path = get_only_path()?;
        let plinezs = get_plinezs!(path);
        let shapezs = compress_heightmap(plinezs, &mut logger);
        println!("Compressed: {} ms", timer.elapsed().as_millis());
        let infos = info_package(&shapezs);
        let buffer = shapezs.compress(infos, &mut logger);
        println!("Bufferized: {} ms", timer.elapsed().as_millis());
        write_buffer(&outfile, &buffer, &timer);
    }else if mode == "xmltree"{// print out the xml open tags in indented tree form.
        for file in infiles{
            println!("\t File: {}", file);
            print_xml_tag_tree(&file);
        }
    }else if mode == "xmltags"{// print out every xml tag with its count.
        for file in infiles{
            println!("\t File: {}", file);
            print_xml_tag_count(&file);
        }
    }else if mode == "geomerge"{
        let mut styles = Vec::new();
        let mut counter = 0;
        let mut polyzs = Vec::new();
        for file in infiles{
            let polys = kml_geo(&file, &mut styles, &mut counter, &mut logger);
            let stpolyzs: Vec<_> = polys.into_iter().map(|(sty,poly)| PolygonZ::from(poly,sty)).collect();
            polyzs.extend(stpolyzs);
        }
        let mut polyzs = polyzs.into_iter().map(|p| int_cast(p)).collect::<Vec<_>>();
        polyzs.iter_mut().for_each(|p| p.stretch_bb());
        println!("There are {} polygons!", polyzs.len());
        let gbb = get_global_bb(&polyzs);
        let cuts = 8u8;
        let triangles = crate::triangulate::triangulate(polyzs, &mut logger);
        let chunks = crate::chunkify::chunkify_polytriangles(cuts, gbb, triangles);
        for (x,y,chunk) in chunks{
            let infos = info_package(&chunk);
            let buffer = chunk.compress(infos, &mut logger);
            let filename = &format!("{}-{}.polychunk", x, y);
            write_buffer(filename, &buffer, &timer);
        }
        let mut stylebuffer = Vec::new();
        styles.into_buffer(&mut stylebuffer);
        write_buffer("styles", &stylebuffer, &timer);
        let mut infobuffer = Vec::new();
        gbb.into_buffer(&mut infobuffer);
        cuts.into_buffer(&mut infobuffer);
        write_buffer("chunks.polyinfo", &infobuffer, &timer);
    }else if mode == "check-tag-child"{
        for file in infiles{
            println!("{}", check_tag_child(&file,&tag0,&tag1));
        }
    }else if mode == "checK-nonempty-tag"{
        for file in infiles{
            println!("{}", check_nonempty_tag(&file,&tag0));
        }
    }else{
        println!("Unsupported mode!");
    }
    logger.report();
    Option::Some(())
}
