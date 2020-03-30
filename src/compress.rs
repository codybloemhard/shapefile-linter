use bin_buffer::*;

use crate::data::{ShapeZ,P3,VvP4};
use crate::logger::*;

pub trait FromU64{
    fn from(x: u64) -> Self;
}

impl FromU64 for u8{ fn from(x: u64) -> Self{ x as u8 } }
impl FromU64 for u16{ fn from(x: u64) -> Self{ x as u16 } }
impl FromU64 for u32{ fn from(x: u64) -> Self{ x as u32 } }

pub trait OffScaleFromU64{
    fn offscale(x: f64, o: u64, m: u64) -> Self;
}

impl<T: FromU64> OffScaleFromU64 for T{
    fn offscale(x: f64, o: u64, m: u64) -> Self{
        T::from(((x - o as f64) * m as f64).round() as u64)
    }
}

fn bb_to_t<T: FromU64>(bb: (P3<f64>,P3<f64>)) -> (P3<T>,P3<T>){
    (
        (T::from((bb.0).0 as u64), T::from((bb.0).1 as u64), T::from((bb.0).2 as u64)),
        (T::from((bb.1).0 as u64), T::from((bb.1).1 as u64), T::from((bb.1).2 as u64)),
    )
}

// pub trait Compressable
// {
//     fn compress<T: Bufferable + FromU64>
//         (self, mx: u64, my: u64, multi: u64) -> U<T>;
// }
//
// impl Compressable<ShapeZ,f64> for ShapeZ<f64>{
//     fn compress<T: Bufferable + FromU64>
//         (self, mx: u64, my: u64, multi: u64) -> ShapeZ<T>{
//
//         }
// }

pub fn compress_shapez_into<T: Bufferable + FromU64>
    (shapezs: Vec<ShapeZ<f64>>, mx: u64, my: u64, multi: u64) -> Vec<ShapeZ<T>>{
    let mut nshapezs = Vec::new();
    for shape in shapezs{
        let mut vec = Vec::new();
        for (x,y) in shape.points{
            let xx = T::offscale(x, mx, multi);
            let yy = T::offscale(y, my, multi);
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

pub fn compress_heightmap(shapes: VvP4, logger: &mut Logger)
    -> Vec<ShapeZ<f64>>{
    let mut shapezs = Vec::new();
    'outer: for shape in shapes{
        if shape.is_empty(){
            logger.log(Issue::EmptyShape);
            continue;
        }
        let mut npoints = Vec::new();
        let z = shape[0].2;
        for point in shape{
            if (point.2 - z).abs() > std::f64::EPSILON{
                logger.log(Issue::TwoPlusZInHeightline);
                continue 'outer;
            }
            npoints.push((point.0,point.1));
        }
        let bb = ((0.0,0.0,0.0),(0.0,0.0,0.0));
        shapezs.push(ShapeZ{
            points: npoints,
            z,
            bb,
        });
    }
    shapezs
}

