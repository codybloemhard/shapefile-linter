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
        }else if mode == "polygonZ"{
            let polys = split(shapes, &mut logger).11;
            let polyzs: Vec<PolygonZ<f64>> = polys.into_iter().map(|x| PolygonZ::from(x)).collect();
            let ranges = compress_doubles_stats(&polyzs);
            let (mx,rx,my,ry) = ranges;
            println!("minx: {}, rangex:{}, miny: {}, rangey: {}", mx, rx, my, ry);
            let shapesrange = compress_shapes_stats(&polyzs);
            println!("shaperangex: {}, shaperangey: {}", shapesrange.0, shapesrange.1);
            let counts = compress_repeated_points_in_lines_stats(&polyzs);
            println!("total: {}, repeated: {}", counts.0, counts.1);
            let (range,target)= target_compression_type(ranges);
            let (multi,usage) = target_multiplier(range,target);
            println!("target {} with multiplier {} using {} of range",
                     target.to_string(), multi, usage);
            let mut buffer = Vec::new();
            polyzs.into_buffer(&mut buffer); // cant bufferize (T,T,T,T)
            println!("Bufferized: {} ms", timer.elapsed().as_millis());
            let ok = buffer_write_file(&Path::new(&outfile), &buffer);
            println!("Writing file \"{}\", went ok?: {}, {} ms", outfile, ok,
                     timer.elapsed().as_millis());
            logger.report();
        }else if mode == "height"{
            let all = split(shapes, &mut logger);
            let plinezs = all.5;
            let mut shapezs = compress_heightmap(plinezs, &mut logger);
            println!("Compressed: {} ms", timer.elapsed().as_millis());
            let ranges = compress_doubles_stats(&shapezs);
            let (mx,rx,my,ry)= ranges;
            println!("minx: {}, rangex:{}, miny: {}, rangey: {}", mx, rx, my, ry);
            let shapesrange = compress_shapes_stats(&shapezs);
            println!("shaperangex: {}, shaperangey: {}", shapesrange.0, shapesrange.1);
            let counts = compress_repeated_points_in_lines_stats(&shapezs);
            println!("total: {}, repeated: {}", counts.0, counts.1);
            let (range,target)= target_compression_type(ranges);
            let (multi,usage) = target_multiplier(range,target);
            println!("target {} with multiplier {} using {} of range",
                     target.to_string(), multi, usage);
            let mut buffer = Vec::new();
            (mx,my,multi).into_buffer(&mut buffer);
            macro_rules! TargetIntoBuffer {
                ($ttype:ident) => {
                    let mut ns = compress_shapez_into::<$ttype>(shapezs,mx,my,multi);
                    let bb = set_bb(&mut ns);
                    bb.into_buffer(&mut buffer);
                    ns.into_buffer(&mut buffer);
                };
            }
            match target{
                CompTarget::U8 => { TargetIntoBuffer!(u8); },
                CompTarget::U16 => { TargetIntoBuffer!(u16); },
                CompTarget::U32 => { TargetIntoBuffer!(u32); },
                CompTarget::NONE => {
                    let bb = set_bb(&mut shapezs);
                    bb.into_buffer(&mut buffer);
                    shapezs.into_buffer(&mut buffer);
                },
            }
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

