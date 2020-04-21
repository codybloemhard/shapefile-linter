use std::collections::HashMap;
use crate::logger::*;
use super::data::*;
use shapefile::*;

pub type Ranges = (u64,u64,u64,u64,u64,u64);

pub fn info_package<'a,S: CustomShape>(shapes: &'a [S]) -> (u64,u64,u64,u64,CompTarget)
    where
        for<'b> &'b S: IntoIterator,
        <&'a S as IntoIterator>::Item: HasXyz<f64> + PartialEq,
{
    let ranges = compress_doubles_stats(shapes);
    let (mx,rx,my,ry,mz,rz) = ranges;
    println!("minx: {}, rangex:{}, miny: {}, rangey: {}, minz: {}, rangez: {}", mx, rx, my, ry, mz, rz);
    let shapesrange = compress_shapes_stats(shapes);
    println!("shaperangex: {}, shaperangey: {}", shapesrange.0, shapesrange.1);
    let counts = compress_repeated_points_in_lines_stats(shapes);
    println!("total: {}, repeated: {}", counts.0, counts.1);
    let (range,target) = target_compression_type(ranges);
    let (multi,usage) = target_multiplier(range,target);
    println!("target {} with multiplier {} using {} of range", target.to_string(), multi, usage);
    (mx,my,mz,multi,target)
}

// using this magic: https://doc.rust-lang.org/nomicon/hrtb.html
pub fn compress_doubles_stats<'a,S>(shapes: &'a [S]) -> Ranges
    where
        for<'b> &'b S: IntoIterator,
        <&'a S as IntoIterator>::Item: HasXyz<f64>,
{
    let mut xmin = std::u64::MAX;
    let mut xmax = std::u64::MIN;
    let mut ymin = std::u64::MAX;
    let mut ymax = std::u64::MIN;
    let mut zmin = std::u64::MAX;
    let mut zmax = std::u64::MIN;
    for shape in shapes{
        for p in shape{
            let xyz = p.xyz();
            let x = xyz.0 as u64;
            let y = xyz.1 as u64;
            let z = xyz.2 as u64;
            xmax = xmax.max(x);
            ymax = ymax.max(y);
            zmax = zmax.max(z);
            xmin = xmin.min(x);
            ymin = ymin.min(y);
            zmin = zmin.min(z);
        }
    }
    (xmin, xmax - xmin, ymin, ymax - ymin, zmin, zmax - zmin)
}

pub fn compress_shapes_stats<'a,S>(shapes: &'a [S]) -> (u64,u64,u64)
    where
        S: CustomShape,
        for<'b> &'b S: IntoIterator,
        <&'a S as IntoIterator>::Item: HasXyz<f64>,
{
    let mut rangex = std::u64::MIN;
    let mut rangey = std::u64::MIN;
    let mut rangez = std::u64::MIN;
    for shape in shapes{
        if shape.points_len() == 0 {
            continue;
        }
        let mut xmin = std::u64::MAX;
        let mut xmax = std::u64::MIN;
        let mut ymin = std::u64::MAX;
        let mut ymax = std::u64::MIN;
        let mut zmin = std::u64::MAX;
        let mut zmax = std::u64::MIN;
        for p in shape{
            let xyz = p.xyz();
            let x = xyz.0 as u64;
            let y = xyz.1 as u64;
            let z = xyz.2 as u64;
            xmax = xmax.max(x);
            ymax = ymax.max(y);
            zmax = zmax.max(z);
            xmin = xmin.min(x);
            ymin = ymin.min(y);
            zmin = zmin.min(z);
        }
        rangex = rangex.max(xmax - xmin);
        rangey = rangey.max(ymax - ymin);
        rangez = rangez.max(zmax - zmin);
    }
    (rangex,rangey,rangez)
}

pub fn compress_repeated_points_in_lines_stats<'a, S>(shapes: &'a [S]) -> (usize,usize)
    where
        for<'b> &'b S: IntoIterator,
        <&'a S as IntoIterator>::Item: PartialEq,
{
    let mut points = 0;
    let mut repeated = 0;
    for shape in shapes{
        let mut iter = shape.into_iter();
        let first = iter.next();
        if first.is_none() { continue; }
        let last = first.unwrap();
        for p in iter{
            points += 1;
            if p == last{
                repeated += 1;
            }
        }
    }
    (points,repeated)
}

#[derive(Copy,Clone)]
pub enum CompTarget{
    U8,U16,U32,NONE,
}

impl std::fmt::Display for CompTarget{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        write!(f,"{}",match self{
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::NONE => "none",
        })
    }
}

pub fn target_compression_type((_,rx,_,ry,_,rz): Ranges) -> (u64, CompTarget){
    fn get_target(range: u64) -> CompTarget{
        if range < std::u8::MAX.into(){ CompTarget::U8 }
        else if range < std::u16::MAX.into(){ CompTarget::U16 }
        else if range < std::u32::MAX.into(){ CompTarget::U32 }
        else {CompTarget::NONE }
    }
    let max = rx.max(ry).max(rz);
    (max,get_target(max))
}

pub fn target_multiplier(mr: u64, target: CompTarget) -> (u64,f64){
    let max: u64 = match target{
        CompTarget::U8 => std::u8::MAX.into(),
        CompTarget::U16 => std::u16::MAX.into(),
        CompTarget::U32 => std::u32::MAX.into(),
        _ => std::u64::MAX,
    };
    let m = max / mr;
    if m < 1 { panic!("Error: target_multiplier smaller than one!"); }
    (m,(m * mr) as f64 / max as f64)
}

pub fn print_height_distribution<T>(shapes: &[ShapeZ<T>])
    where
        T: std::hash::Hash + std::cmp::Eq + std::fmt::Display + std::cmp::Ord
{
    println!("Shapes: {}", shapes.len());
    let mut distr = HashMap::new();
    for shape in shapes{
        let z = &shape.z;
        let nc = match distr.get(&z){
            Some(count) => count + 1,
            None => 1,
        };
        distr.insert(z, nc);
    }
    let mut vec = Vec::new();
    for item in &distr{
        vec.push(item);
    }
    vec.sort_by_key(|x| x.0);
    for (z,c) in vec{
        print!("{}: {}, ", z, c);
    }
    println!();
}

pub fn collect_wrong_heightlines(shapes: VvP4, logger: &mut Logger)
    -> Vvec<f64>
{
    let mut wrong = Vec::new();
    for shape in shapes{
        if shape.is_empty(){
            logger.log(Issue::EmptyShape);
            continue;
        }
        let mut is_wrong = false;
        let z = shape[0].2;
        for point in &shape{
            if (point.2 - z).abs() > std::f64::EPSILON{
                is_wrong = true;
                break;
            }
        }
        if !is_wrong { continue; }
        let mut vec = Vec::new();
        for point in shape{
            vec.push(point.2);
        }
        wrong.push(vec);
    }
    wrong
}

pub fn print_split_content((ps,pms,pzs,pls,plms,plzs,mps,mpms,mpzs,pgs,pgms,pgzs): &Splitted){
    println!("How much of everything is present in this shapefile: ");
    println!("Point's: {}", ps.len());
    println!("PointM's: {}", pms.len());
    println!("PointZ's: {}", pzs.len());
    println!("Polyline's: {}", pls.len());
    println!("PolylineM's: {}", plms.len());
    println!("PolylineZ's: {}", plzs.len());
    println!("Multipoint's: {}", mps.len());
    println!("MultipointM's: {}", mpms.len());
    println!("MultipointZ's: {}", mpzs.len());
    println!("Polygon's: {}", pgs.len());
    println!("PolygonM's: {}", pgms.len());
    println!("PolygonZ's: {}", pgzs.len());
}

pub fn print_shape_content(shapes: &[Shape]){
    let mut p = 0; let mut pm = 0; let mut pz = 0;
    let mut pl = 0; let mut plp = 0; let mut ps = 0;
    let mut plm = 0; let mut plmp = 0; let mut plz = 0;
    let mut plzp = 0; let mut pg = 0; let mut pgp = 0;
    let mut pgm = 0; let mut pgmp = 0; let mut pgz = 0;
    let mut pgzp = 0; let mut mp = 0; let mut mpm = 0;
    let mut mpz = 0; let mut ma = 0; let mut map = 0;
    let mut ns = 0;
    for shape in shapes{
        match shape{
            Shape::NullShape => { ns += 1; }
            Shape::Point(_) => { p += 1; },
            Shape::PointM(_) => { pm += 1; },
            Shape::PointZ(_) => { pz += 1; },
            Shape::Polyline(x) => { pl += 1; plp += x.parts().len(); ps += x.total_point_count(); },
            Shape::PolylineM(x) => { plm += 1; plmp += x.parts().len(); ps += x.total_point_count(); },
            Shape::PolylineZ(x) => { plz += 1; plzp += x.parts().len(); ps += x.total_point_count(); },
            Shape::Polygon(x) => { pg += 1; pgp += x.rings().len(); ps += x.total_point_count(); },
            Shape::PolygonM(x) => { pgm += 1; pgmp += x.rings().len(); ps += x.total_point_count(); },
            Shape::PolygonZ(x) => { pgz += 1; pgzp += x.rings().len(); ps += x.total_point_count(); },
            Shape::Multipoint(x) => { mp += 1; ps += x.points().len(); },
            Shape::MultipointM(x) => { mpm += 1; ps += x.points().len(); },
            Shape::MultipointZ(x) => { mpz += 1; ps += x.points().len(); },
            Shape::Multipatch(x) => { ma += 1; map += x.patches().len(); ps += x.total_point_count(); },
        }
    }
    println!("Total points: {}", ps + p + pm + pz);
    println!("Null shapes: {}", ns);
    println!("Point's: {}", p);
    println!("PointM's: {}", pm);
    println!("PointZ's: {}", pz);
    println!("Polyline's: {}, with total parts: {}", pl, plp);
    println!("PolylineM's: {}, with total parts: {}", plm, plmp);
    println!("PolylineZ's: {}, with total parts: {}", plz, plzp);
    println!("Polygon's: {}, with total rings: {}", pg, pgp);
    println!("PolygonM's: {}, with total rings: {}", pgm, pgmp);
    println!("PolygonZ's: {}, with total rings: {}", pgz, pgzp);
    println!("Multipoint's: {}", mp);
    println!("MultipointM's: {}", mpm);
    println!("MultipointZ's: {}", mpz);
    println!("Multipatch's: {}, with patches: {}", ma, map);
}
