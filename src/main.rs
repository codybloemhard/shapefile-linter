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

use data::*;
use info::*;
use compress::*;
use logger::*;

fn main() {
    let args = lapp::parse_args("
    Preprocess shapefiles into more efficient files.
      <inputfile> (string) input file name
      <outputfile> (string) output file name"
    );

    let infile = args.get_string("inputfile");
    let outfile = args.get_string("outputfile");

    let mut logger = Logger::new();

    println!("Shapefile processor...");
    let timer = Instant::now();
    if let Ok(shapes) = shapefile::read(infile.clone()){
        println!("Read file \"{}\": {} ms", infile, timer.elapsed().as_millis());
        println!("Shapes: {}", shapes.len());
        let mut shapezs = compress_heightmap(shapes, &mut logger);
        println!("Compressed: {} ms", timer.elapsed().as_millis());
        let ranges = compress_doubles_stats(&shapezs);
        let (mx,rx,my,ry)= ranges;
        println!("minx: {}, rangex:{}, miny: {}, rangey: {}", mx, rx, my, ry);
        let shapesrange = compress_shapes_stats(&shapezs);
        println!("shaperangex: {}, shaperangey: {}", shapesrange.0, shapesrange.1);
        let counts = compress_repeated_points_in_lines_stats(&shapezs);
        println!("total: {}, repeated: {}", counts.0, counts.1);
        let target = target_compression_type(ranges);
        println!("target {}", target.to_string());
        let mut buffer = Vec::new();
        (mx,my).into_buffer(&mut buffer);
        macro_rules! TargetIntoBuffer {
            ($ttype:ident,$buffer:ident,$shapezs:ident,$ranges:ident) => {
                let mut ns = compress_shapez_into::<$ttype>($shapezs, $ranges);
                let bb = set_bb(&mut ns);
                bb.into_buffer(&mut $buffer);
                ns.into_buffer(&mut $buffer);
            };
        }
        match target{
            CompTarget::U8 =>
                {TargetIntoBuffer!(u8,buffer,shapezs,ranges);},
            CompTarget::U16 =>
                {TargetIntoBuffer!(u16,buffer,shapezs,ranges);},
            CompTarget::U32 =>
                {TargetIntoBuffer!(u32,buffer,shapezs,ranges);},
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
        println!("Could not read file: {}", infile);
    }
}


