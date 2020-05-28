use bin_buffer::*;
use crate::data::{PolygonZ,Vvec,StretchableBB,get_global_bb,UpdateableBB,ShapeZ,P3,VvP4};
use crate::info::CompTarget;
use crate::logger::*;
use crate::triangulate::triangulate;
use crate::triangulate::PolyTriangle;
// To get a value from u64, std converts are safe
// I know when i can cast(from offset, multi, etc)
// Just want to do it without checking
pub trait FromU64{
    fn from(x: u64) -> Self;
}

impl FromU64 for u8{ fn from(x: u64) -> Self{ x as u8 } }
impl FromU64 for u16{ fn from(x: u64) -> Self{ x as u16 } }
impl FromU64 for u32{ fn from(x: u64) -> Self{ x as u32 } }

pub trait FromF64{
    fn from(x: f64) -> Self;
}

impl FromF64 for u8{ fn from(x: f64) -> Self{ x as u8 } }
impl FromF64 for u16{ fn from(x: f64) -> Self{ x as u16 } }
impl FromF64 for u32{ fn from(x: f64) -> Self{ x as u32 } }
impl FromF64 for u64{ fn from(x: f64) -> Self{ x as u64 } }
impl FromF64 for f32{ fn from(x: f64) -> Self{ x as f32 } }
impl FromF64 for f64{ fn from(x: f64) -> Self{ x as f64 } }

// Peform compression by using a range offset and multiplier
pub trait OffScaleFromU64{
    fn offscale(x: f64, o: u64, m: u64) -> Self;
}
// Generic implementation
impl<T: FromU64> OffScaleFromU64 for T{
    fn offscale(x: f64, o: u64, m: u64) -> Self{
        T::from(((x - o as f64) * m as f64).round() as u64)
    }
}
// We must be able to cast a bounding box to the right type as well
fn bb_to_t<T: FromU64>(bb: (P3<f64>,P3<f64>)) -> (P3<T>,P3<T>){
    (
        (T::from((bb.0).0 as u64), T::from((bb.0).1 as u64), T::from((bb.0).2 as u64)),
        (T::from((bb.1).0 as u64), T::from((bb.1).1 as u64), T::from((bb.1).2 as u64)),
    )
}
// Ability to be convertable to a compressed buffer
pub trait Compressable
{
    fn compress(self, infos: (u64,u64,u64,u64,CompTarget), logger: &mut Logger) -> Buffer;
}
// Ability to be transformed into triangles and than compressed into buffer
pub trait TriangleCompressable{
    fn triangle_compress(self, infos: (u64,u64,u64,u64,CompTarget), logger: &mut Logger) -> Buffer;
}
// Macro that builds a generic implementation of Compressable
macro_rules! ImplCompressable {
    ($tname:ident,$tfname:ident,$btype:ty,$fname:ident,$trans:ident) => {
        impl $tname for $btype
        {
            fn $tfname
                (mut self, (mx,my,mz,multi,target): (u64,u64,u64,u64,CompTarget), logger: &mut Logger) -> Buffer{
                    let mut buffer = Vec::new();
                    (mx,my,mz,multi).into_buffer(&mut buffer);
                    macro_rules! TargetIntoBuffer {
                        ($ttype:ident) => {
                            let mut ns = $fname::<$ttype>(self,mx,my,multi);
                            ns.iter_mut().for_each(|x| x.stretch_bb());
                            ns.iter_mut().for_each(|x| x.update_bb());
                            let bb = get_global_bb(&ns);
                            println!("Global Boundingbox: {:?}", bb);
                            // transform?
                            let ns = $trans(ns,logger);
                            bb.into_buffer(&mut buffer);
                            ns.into_buffer(&mut buffer);
                        };
                    }
                    match target{
                        CompTarget::U8 => { TargetIntoBuffer!(u8); },
                        CompTarget::U16 => { TargetIntoBuffer!(u16); },
                        CompTarget::U32 => { TargetIntoBuffer!(u32); },
                        CompTarget::NONE => {
                            self.iter_mut().for_each(|x| x.stretch_bb());
                            let bb = get_global_bb(&self);
                            bb.into_buffer(&mut buffer);
                            self.into_buffer(&mut buffer);
                        },
                    }
                    buffer
                }
        }
    };
}
fn id<T>(v: T, _logger: &mut Logger) -> T { v }
// Actually implement it for the needed types
ImplCompressable!(Compressable,compress,Vec<ShapeZ<f64>>,compress_shapez_into,id);
ImplCompressable!(Compressable,compress,Vec<PolygonZ<f64>>,compress_polygonz_into,id);
ImplCompressable!(Compressable,compress,Vec<PolyTriangle<u32>>,compress_polytriangle_into,id);
ImplCompressable!(TriangleCompressable,triangle_compress,Vec<PolygonZ<f64>>,compress_polygonz_into,triangulate);
// Take ShapeZ of f64 and turn into ShapeZ of given T
// Used to implement Compressable
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
// Same as above but for PolygonZ
pub fn compress_polygonz_into<T: Bufferable + FromU64>
    (polygonzs: Vec<PolygonZ<f64>>, mx: u64, my: u64, multi: u64) -> Vec<PolygonZ<T>>{
    let mut npolygonzs = Vec::new();
    for pz in polygonzs{
        let style = pz.style;
        let build = |old: Vvec<P3<f64>>|{
            let mut col = Vec::new();
            for sub in old{
                let mut vec = Vec::new();
                for (x,y,z) in sub{
                    let xx = T::offscale(x, mx, multi);
                    let yy = T::offscale(y, my, multi);
                    let zz = T::offscale(z, 0, 0); //TODO
                    vec.push((xx,yy,zz));
                }
                col.push(vec);
            }
            col
        };
        npolygonzs.push(PolygonZ{
            inners: build(pz.inners),
            outers: build(pz.outers),
            bb: bb_to_t::<T>(pz.bb),
            style,
        });
    }
    npolygonzs
}
pub fn compress_polytriangle_into<T: Bufferable + FromU64>
    (polytriangles: Vec<PolyTriangle<u32>>, mx: u64, my: u64, multi: u64) -> Vec<PolyTriangle<T>>
{
    let mut npts = Vec::new();
    for pt in polytriangles{
        let mut vec = Vec::new();
        for (x,y) in pt.vertices{
            let xx = T::offscale(x as f64, mx, multi);
            let yy = T::offscale(y as f64, my, multi);
            vec.push((xx,yy));
        }
        let ((a,b,c),(d,e,f)) = pt.bb;
        let fbb = ((a as f64, b as f64, c as f64),(d as f64, e as f64, f as f64));
        npts.push(PolyTriangle{
            vertices: vec,
            indices: pt.indices,
            style: pt.style,
            bb: bb_to_t::<T>(fbb),
        });
    }
    npts
}
// Specific commpression method for heightmaps
// Does nothing with value types, removes redundant data
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

