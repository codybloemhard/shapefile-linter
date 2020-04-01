extern crate lapp;
extern crate shapefile;
extern crate bin_buffer;

use bin_buffer::*;
use std::path::Path;
use std::time::Instant;

pub mod data;
pub mod info;
pub mod compress;
pub mod logger;

use info::*;
use compress::*;
use logger::*;
use data::*;
use crate::data::{PolygonZ};

fn main(){
    do_things();
}

fn do_things() -> Option<()>{
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
    let read_single_file = |infile: String|{
        if let Ok(shapes) = shapefile::read(infile.clone()){
            println!("Read file \"{}\": {} ms", infile, timer.elapsed().as_millis());
            println!("Shapes: {}", shapes.len());
            Option::Some(shapes)
        }else{
            println!("Could not read file: {}", infile);
            Option::None
        }
    };
    let read_only_file = ||{
        if infiles.is_empty() { return Option::None; }
        read_single_file(infiles[0].clone())
    };
    if mode == "info"{
        let shapes = read_only_file()?;
        print_shape_content(&shapes);
        let splitted = split(shapes, &mut logger);
        print_split_content(&splitted);
    }else if mode == "merge"{
        println!("{:?}", infiles);
    }else if mode == "polygonz"{
        let shapes = read_only_file()?;
        let polys = split(shapes, &mut logger).11;
        let polyzs: Vec<PolygonZ<f64>> = polys.into_iter().map(PolygonZ::from).collect();
        let infos = info_package(&polyzs);
        let buffer = polyzs.compress(infos);
        println!("Bufferized: {} ms", timer.elapsed().as_millis());
        let ok = buffer_write_file(&Path::new(&outfile), &buffer);
        println!("Writing file \"{}\", went ok?: {}, {} ms", outfile, ok,
                 timer.elapsed().as_millis());
        logger.report();
    }else if mode == "height"{
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
        logger.report();
    }else{
        println!("Unsupported mode!");
    }
    Option::Some(())
}
