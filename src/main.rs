extern crate lapp;
extern crate shapefile;
extern crate bin_buffer;

use shapefile::*;
use bin_buffer::*;
use std::path::Path;
use std::time::Instant;

fn main() {
    let args = lapp::parse_args("
    Preprocess shapefiles into more efficient files.
      <inputfile> (string) input file name
      <outputfile> (string) output file name"
    );

    let infile = args.get_string("inputfile");
    let outfile = args.get_string("outputfile");

    println!("Shapefile processor...");
    let timer = Instant::now();
    if let Ok(shapes) = shapefile::read(infile.clone()){
        println!("Read file \"{}\": {} ms", infile, timer.elapsed().as_millis());
        println!("Shapes: {}", shapes.len());
        let shapezs = compress_heightmap(shapes);
        println!("Compressed: {} ms", timer.elapsed().as_millis());
        let ranges = compress_doubles_stats(&shapezs);
        let (mx,rx,my,ry)= ranges;
        println!("minx: {}, rangex:{}, miny: {}, rangey: {}", mx, rx, my, ry);
        let target = target_compression_type(ranges);
        println!("target {}", target.to_string());
        let mut buffer = Vec::new();
        mx.into_buffer(&mut buffer);
        my.into_buffer(&mut buffer);
        match target{
            CompTarget::U8 => compress_shapez_into::<u8>(shapezs, ranges).into_buffer(&mut buffer),
            CompTarget::U16 => compress_shapez_into::<u16>(shapezs, ranges).into_buffer(&mut buffer),
            CompTarget::U32 => compress_shapez_into::<u32>(shapezs, ranges).into_buffer(&mut buffer),
            CompTarget::NONE => shapezs.into_buffer(&mut buffer),
        }
        println!("Bufferized: {} ms", timer.elapsed().as_millis());
        let ok = buffer_write_file(&Path::new(&outfile), &buffer);
        println!("Writing file \"{}\", went ok?: {}, {} ms", outfile, ok,
                 timer.elapsed().as_millis());
    }else{
        println!("Could not read file: {}", infile);
    }
}

type P2<T> = (T,T);
type P3 = (f64,f64,f64);

#[derive(Clone)]
struct ShapeZ<T>{
    points: Vec<P2<T>>,
    z: f64,
    bb: (P3,P3),
}

impl<T: Bufferable + Clone> Bufferable for ShapeZ<T>{
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
            let p = if let Some(wp) = <P2<T>>::from_buffer(buf){ wp }
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

type Ranges = (u64,u64,u64,u64);

fn compress_doubles_stats(shapezs: &Vec<ShapeZ<f64>>) -> Ranges{
    let mut xmin = std::u64::MAX;
    let mut xmax = std::u64::MIN;
    let mut ymin = std::u64::MAX;
    let mut ymax = std::u64::MIN;
    for shape in shapezs{
        for p in &shape.points{
            let i0 = p.0 as u64;
            let i1 = p.1 as u64;
            xmax = xmax.max(i0);
            ymax = ymax.max(i1);
            xmin = xmin.min(i0);
            ymin = ymin.min(i1);
        }
    }
    (xmin, xmax - xmin, ymin, ymax - ymin)
}

#[derive(Copy,Clone)]
enum CompTarget{
    U8,U16,U32,NONE,
}

impl CompTarget{
    fn to_string(self) -> String{
        match self{
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::NONE => "none",
        }.to_string()
    }
}

fn target_compression_type((_,rx,_,ry): Ranges) -> CompTarget{
    fn get_target(range: u64) -> CompTarget{
        if range < std::u8::MAX.into(){ CompTarget::U8 }
        else if range < std::u16::MAX.into(){ CompTarget::U16 }
        else if range < std::u32::MAX.into(){ CompTarget::U32 }
        else {CompTarget::NONE }
    }
    get_target(rx.max(ry))
}

trait FromU64{
    fn from(x: u64) -> Self;
}

impl FromU64 for u8{ fn from(x: u64) -> Self{ x as u8 } }
impl FromU64 for u16{ fn from(x: u64) -> Self{ x as u16 } }
impl FromU64 for u32{ fn from(x: u64) -> Self{ x as u32 } }

fn compress_shapez_into<T: Bufferable + FromU64>
    (shapezs: Vec<ShapeZ<f64>>, (mx,_,my,_): Ranges) -> Vec<ShapeZ<T>>{
    let mut nshapezs = Vec::new();
    for shape in shapezs{
        let mut vec = Vec::new();
        for (x,y) in shape.points{
            let xx = T::from((x as u64) - mx);
            let yy = T::from((y as u64) - my);
            vec.push((xx,yy));
        }
        nshapezs.push(ShapeZ{
            points: vec,
            z: shape.z,
            bb: shape.bb,
        });
    }
    nshapezs
}

fn compress_heightmap(shapes: Vec<Shape>) -> Vec<ShapeZ<f64>>{
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

