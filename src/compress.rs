use bin_buffer::*;
use shapefile::*;

use crate::data::ShapeZ;
use crate::info::Ranges;

pub trait FromU64{
    fn from(x: u64) -> Self;
}

impl FromU64 for u8{ fn from(x: u64) -> Self{ x as u8 } }
impl FromU64 for u16{ fn from(x: u64) -> Self{ x as u16 } }
impl FromU64 for u32{ fn from(x: u64) -> Self{ x as u32 } }

pub fn compress_shapez_into<T: Bufferable + FromU64>
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

