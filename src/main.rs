extern crate lapp;
extern crate shapefile;
extern crate bin_buffer;
extern crate dlv_list;

use bin_buffer::*;
use std::path::Path;
use std::time::Instant;

pub mod data;
pub mod info;
pub mod compress;
pub mod logger;
pub mod chunkify;
pub mod triangulate;

use info::*;
use compress::*;
use logger::*;
use data::*;
use crate::data::{PolygonZ};
use chunkify::*;
use triangulate::*;

fn main(){
    do_things();
}

fn do_things() -> Option<()>{
    // Set up cli arguments
    let args = lapp::parse_args("
    Preprocess shapefiles into more efficient files.
      <inputfile> (string...) input file(s) name(s)
      --output (default outp) (string) define output file
      --mode (default info) (string) what to do"
    );

    let infiles = args.get_strings("inputfile");
    let outfile = args.get_string("output");
    let mode = args.get_string("mode");

    let mut logger = Logger::default();

    println!("Shapefile processor...");
    let timer = Instant::now();
    // Take one file
    let get_only_path = ||{
        if infiles.is_empty() { return Option::None; }
        Option::Some(infiles[0].clone())
    };
    // Take all files
    let read_single_file = |infile: String|{
        if let Ok(shapes) = shapefile::read(infile.clone()){
            println!("Read file \"{}\": {} ms, shapes: {}", infile, timer.elapsed().as_millis(), shapes.len());
            Option::Some(shapes)
        }else{
            println!("Could not read file: {}", infile);
            Option::None
        }
    };
    // Read one file, assume it's the only one, more can't hurt.
    let read_only_file = ||{
        if infiles.is_empty() { return Option::None; }
        read_single_file(infiles[0].clone())
    };
    // Compress and bufferize and write collection.
    macro_rules! compress_and_write{
        ($col:expr) =>{
            let infos = info_package(&$col);
            let buffer = $col.compress(infos);
            let ok = buffer_write_file(&Path::new(&outfile), &buffer);
            println!("Writing file \"{}\", went ok?: {}, {} ms", outfile, ok,
                     timer.elapsed().as_millis());
        }
    }
    if mode == "info"{// Just print info about shapefile content.
        let shapes = read_only_file()?;
        print_shape_content(&shapes);
        let splitted = split(shapes, &mut logger);
        print_split_content(&splitted);
    }else if mode == "mergeheight"{// Take many heightfiles and combine them into one big compressed one.
        println!("{:?}", infiles);
        let mut collection = Vec::new();
        for file in infiles{
            let read = read_single_file(file)?;
            let plinezs = split(read, &mut logger).5;
            let mut shapezs = compress_heightmap(plinezs, &mut logger);
            collection.append(&mut shapezs);
        }
        compress_and_write!(collection);
    }else if mode == "lintheight"{// Print info about heightlines
        let mut wrongs = Vec::new();
        for file in infiles{
            let read = read_single_file(file)?;
            let plinezs = split(read, &mut logger).5;
            let mut vec = collect_wrong_heightlines(plinezs, &mut logger);
            wrongs.append(&mut vec);
        }
        println!("There are {} wrong heightlines", wrongs.len());
        let mut diffs = Vec::new();
        for (i,wrong) in wrongs.iter().enumerate(){
            let min = wrong.iter().fold(std::f64::MAX, |m,x| m.min(*x));
            let max = wrong.iter().fold(std::f64::MIN, |m,x| m.max(*x));
            let diff = max - min;
            diffs.push(diff);
            println!("Line {}: min: {} max: {} diff: {}", i, min, max, diff);
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
        println!("Differences between min and max in lines, summary:");
        println!("median: {}", median);
        println!("mean: {}", mean);
        println!("min: {}", min);
        println!("max: {}", max);
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
        print_height_distribution(&shapes);
        let cuts_mul = 2;
        let mut cuts = 1u64;
        let mut info_buffer = Vec::new();
        let levels = 5u64;
        levels.into_buffer(&mut info_buffer);
        let mods = vec![100,50,20,10,1];
        for i in 0..levels{
            for (x,y,chunk) in cut(cuts.max(1), (bmin,bmax), &shapes, &mut logger){
                let mut buffer = Vec::new();
                i.into_buffer(&mut buffer);
                x.into_buffer(&mut buffer);
                y.into_buffer(&mut buffer);
                let filtered = pick_heights(mods[i as usize], chunk);
                let max = if mods[i as usize] == 1 { std::usize::MAX } else { 5000 };
                let picked = pick_points(max, filtered);
                picked.into_buffer(&mut buffer);
                let ok = buffer_write_file(&Path::new(&format!("{}-{}-{}.chunk", i, x, y)), &buffer);
                println!("Writing chunk ({},{},{}) ok?: {}, {} ms", i, x, y, ok, timer.elapsed().as_millis());
                cuts.into_buffer(&mut info_buffer);
            }
            cuts *= cuts_mul;
        }
        mx.into_buffer(&mut info_buffer);
        my.into_buffer(&mut info_buffer);
        mz.into_buffer(&mut info_buffer);
        multi.into_buffer(&mut info_buffer);
        bmin.into_buffer(&mut info_buffer);
        bmax.into_buffer(&mut info_buffer);
        let ok = buffer_write_file(&Path::new("chunks.info"), &info_buffer);
        println!("Writing file \"chunks.info\" ok?: {}", ok);
    }else if mode == "polygonz"{// Take shapefile and compress the polygonZ's into triangles.
        let shapes = read_only_file()?;
        let polys = split(shapes, &mut logger).11;
        let polyzs: Vec<PolygonZ<f64>> = polys.into_iter().map(PolygonZ::from).collect();
        let triangles = triangulate(polyzs);
        // let infos = info_package(&polyzs);
        // let buffer = polyzs.compress(infos);
        // println!("Bufferized: {} ms", timer.elapsed().as_millis());
        // let ok = buffer_write_file(&Path::new(&outfile), &buffer);
        // println!("Writing file \"{}\", went ok?: {}, {} ms", outfile, ok,
        //          timer.elapsed().as_millis());
    }else if mode == "height"{// Compress shapefile, assuming it consist of height lines.
        let shapes = read_only_file()?;
        let all = split(shapes, &mut logger);
        let plinezs = all.5;
        let shapezs = compress_heightmap(plinezs, &mut logger);
        println!("Compressed: {} ms", timer.elapsed().as_millis());
        let infos = info_package(&shapezs);
        let buffer = shapezs.compress(infos);
        println!("Bufferized: {} ms", timer.elapsed().as_millis());
        let ok = buffer_write_file(&Path::new(&outfile), &buffer);
        println!("Writing file \"{}\", went ok?: {}, {} ms", outfile, ok,
                 timer.elapsed().as_millis());
    }else{
        println!("Unsupported mode!");
    }
    logger.report();
    Option::Some(())
}
