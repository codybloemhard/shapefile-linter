extern crate lapp;
extern crate shapefile;
extern crate bin_buffer;

use shapefile::*;
use bin_buffer::*;
use std::path::Path;

fn main() {
    let args = lapp::parse_args("
    Preprocess shapefiles into more efficient files.
      <file> (string) input file name"
    );

    let file = args.get_string("file");

    println!("Shapefile processor...");
    if let Ok(shapes) = shapefile::read(file.clone()){
        println!("Shapes: {}", shapes.len());
        let shapezs = compress_heightmap(shapes);
        let mut buffer = Vec::new();
        shapezs.into_buffer(&mut buffer);
        let ok = buffer_write_file(&Path::new("heightmap.bin"), &buffer);
        println!("Writing file went ok?: {}", ok);
    }else{
        println!("Could not read file: {}", file);
    }
}

type P2 = (f64,f64);
type P3 = (f64,f64,f64);

#[derive(Clone)]
struct ShapeZ{
    points: Vec<P2>,
    z: f64,
    bb: (P3,P3),
}

impl Bufferable for ShapeZ{
    fn into_buffer(self, buf: &mut Buffer){
        self.z.into_buffer(buf);
        self.bb.0.into_buffer(buf);
        self.bb.1.into_buffer(buf);
        self.points.into_buffer(buf);
    }

    fn copy_into_buffer(&self, buf: &mut Buffer){
        self.clone().into_buffer(buf);
    }

    fn from_buffer(buf: &mut ReadBuffer) -> Option<Self>{
        let z = if let Some(wz) = f64::from_buffer(buf){ wz }
        else { return Option::None; };
        let bb0 = if let Some(wbb0) = <P3>::from_buffer(buf){ wbb0 }
        else { return Option::None; };
        let bb1 = if let Some(wbb1) = <P3>::from_buffer(buf){ wbb1 }
        else { return Option::None; };
        let len = if let Some(wlen) = u64::from_buffer(buf){ wlen }
        else { return Option::None; };
        let mut vec = Vec::new();
        for _ in 0..len{
            let p = if let Some(wp) = <P2>::from_buffer(buf){ wp }
            else { return Option::None; };
            vec.push(p);
        }
        Option::Some(Self{
            points: vec,
            z,
            bb: (bb0,bb1),
        })
    }
}

fn compress_heightmap(shapes: Vec<Shape>) -> Vec<ShapeZ>{
    let mut shapezs = Vec::new();
    'outer: for shape in shapes{
        match shape {
            Shape::PolylineZ(polylinez) => {
                if polylinez.parts.len() > 1{
                    println!("Warning: skipped shape, more than 1 part!");
                    continue;
                }
                if polylinez.points.is_empty(){
                    println!("Warning: skipped shape, 0 points!");
                    continue;
                }
                let mut npoints = Vec::new();
                let z = polylinez.points[0].z;
                let mut minx = std::f64::MAX;
                let mut maxx = std::f64::MIN;
                let mut miny = std::f64::MAX;
                let mut maxy = std::f64::MIN;
                let mut minz = std::f64::MAX;
                let mut maxz = std::f64::MIN;
                for point in polylinez.points {
                    if point.z != z{
                        println!("Warning: skipped shape, not all z equal!");
                        continue 'outer;
                    }
                    minx = minx.min(point.x);
                    maxx = maxx.max(point.x);
                    miny = miny.min(point.y);
                    maxy = maxy.max(point.y);
                    minz = minz.min(point.z);
                    maxz = maxz.max(point.z);
                    npoints.push((point.x,point.y));
                }
                let bb = ((minx,miny,minz),(maxx,maxy,maxz));
                shapezs.push(ShapeZ{
                    points: npoints,
                    z,
                    bb,
                });
            },
            _ => { println!("Bruh"); }
        }
    }
    shapezs
}

