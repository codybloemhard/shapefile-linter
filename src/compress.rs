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

