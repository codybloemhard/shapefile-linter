use bin_buffer::*;
use shapefile::*;

use crate::data::{ShapeZ,P3};
use crate::info::Ranges;

pub trait FromU64{
    fn from(x: u64) -> Self;
}

impl FromU64 for u8{ fn from(x: u64) -> Self{ x as u8 } }
impl FromU64 for u16{ fn from(x: u64) -> Self{ x as u16 } }
impl FromU64 for u32{ fn from(x: u64) -> Self{ x as u32 } }

pub trait OffsetFromU64{
    fn offset(x: f64, o: u64) -> Self;
}

impl<T: FromU64> OffsetFromU64 for T{
    fn offset(x: f64, o: u64) -> Self{
        T::from((x as u64) - o)
    }
}

fn bb_to_t<T: FromU64>(bb: (P3<f64>,P3<f64>)) -> (P3<T>,P3<T>){
    (
        (T::from((bb.0).0 as u64), T::from((bb.0).1 as u64), T::from((bb.0).2 as u64)),
        (T::from((bb.1).0 as u64), T::from((bb.1).1 as u64), T::from((bb.1).2 as u64)),
    )
}

pub fn compress_shapez_into<T: Bufferable + FromU64>
    (shapezs: Vec<ShapeZ<f64>>, (mx,_,my,_): Ranges) -> Vec<ShapeZ<T>>{
    let mut nshapezs = Vec::new();
    for shape in shapezs{
        let mut vec = Vec::new();
        for (x,y) in shape.points{
            let xx = T::offset(x, mx);
            let yy = T::offset(y, my);
            vec.push((xx,yy));
        }
        nshapezs.push(ShapeZ{
            points: vec,
            z: T::from(shape.z as u64),
            bb: bb_to_t::<T>(shape.bb),
        });
    }
    nshapezs
}

pub trait MinMax{
    fn minv() -> Self;
    fn maxv() -> Self;
    fn min_of(self, x: Self) -> Self;
    fn max_of(self, x: Self) -> Self;
}

macro_rules! ImplMinMax {
    ($ttype:ident) => {
        impl MinMax for $ttype
        {
            fn minv() -> Self{ std::$ttype::MIN }
            fn maxv() -> Self{ std::$ttype::MAX }
            fn min_of(self, x: Self) -> Self{ self.min(x) }
            fn max_of(self, x: Self) -> Self{ self.max(x) }
        }
    };
}

ImplMinMax!(f64);
ImplMinMax!(f32);
ImplMinMax!(u64);
ImplMinMax!(u32);
ImplMinMax!(u16);
ImplMinMax!(u8);

pub fn set_bb<T: MinMax + Copy>
    (shapes: &mut Vec<ShapeZ<T>>) -> ((T,T,T),(T,T,T)){
    let mut gminx = T::maxv();
    let mut gmaxx = T::minv();
    let mut gminy = T::maxv();
    let mut gmaxy = T::minv();
    let mut gminz = T::maxv();
    let mut gmaxz = T::minv();
    for shape in shapes{
        let mut minx = T::maxv();
        let mut maxx = T::minv();
        let mut miny = T::maxv();
        let mut maxy = T::minv();
        for point in &shape.points{
            minx = minx.min_of(point.0);
            maxx = maxx.max_of(point.0);
            miny = miny.min_of(point.1);
            maxy = maxy.max_of(point.1);
        }
        shape.bb = ((minx,miny,shape.z),(maxx,maxy,shape.z));
        gminx = gminx.min_of(minx);
        gmaxx = gmaxx.max_of(maxx);
        gminy = gminy.min_of(miny);
        gmaxy = gmaxy.max_of(maxy);
        gminz = gminz.min_of(shape.z);
        gmaxz = gmaxz.max_of(shape.z);
    }
    ((gminx,gminy,gminz),(gmaxx,gmaxy,gmaxz))
}

pub fn compress_heightmap(shapes: Vec<Shape>) -> Vec<ShapeZ<f64>>{
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
                for point in polylinez.points {
                    if point.z != z{
                        println!("Warning: skipped shape, not all z equal!");
                        continue 'outer;
                    }
                    npoints.push((point.x,point.y));
                }
                let bb = ((0.0,0.0,0.0),(0.0,0.0,0.0));
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
