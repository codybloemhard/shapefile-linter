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

fn main() {
    let args = lapp::parse_args("
    Preprocess shapefiles into more efficient files.
      <inputfile> (string) input file name
      --output (default outp) (string) define output file
      --mode (default info) (string) what to do"
    );

    let infile = args.get_string("inputfile");
    let outfile = args.get_string("output");
    let mode = args.get_string("mode");

    let mut logger = Logger::default();

    println!("Shapefile processor...");
    let timer = Instant::now();
    if let Ok(shapes) = shapefile::read(infile.clone()){
        println!("Read file \"{}\": {} ms", infile, timer.elapsed().as_millis());
        println!("Shapes: {}", shapes.len());
        if mode == "info"{
            print_shape_content(&shapes);
            let splitted = split(shapes, &mut logger);
            print_split_content(&splitted);
        }else if mode == "polygonz"{
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
    }else{
        println!("Could not read file: {}", infile);
    }
}
