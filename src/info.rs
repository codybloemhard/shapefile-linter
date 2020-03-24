use super::data::*;
use shapefile::*;

pub type Ranges = (u64,u64,u64,u64);

pub fn compress_doubles_stats(shapezs: &[ShapeZ<f64>]) -> Ranges{
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

pub fn compress_shapes_stats(shapezs: &[ShapeZ<f64>]) -> (u64,u64){
    let mut rangex = std::u64::MIN;
    let mut rangey = std::u64::MIN;
    for shape in shapezs{
        let mut xmin = std::u64::MAX;
        let mut xmax = std::u64::MIN;
        let mut ymin = std::u64::MAX;
        let mut ymax = std::u64::MIN;
        for p in &shape.points{
            let i0 = p.0 as u64;
            let i1 = p.1 as u64;
            xmax = xmax.max(i0);
            ymax = ymax.max(i1);
            xmin = xmin.min(i0);
            ymin = ymin.min(i1);
        }
        rangex = rangex.max(xmax - xmin);
        rangey = rangey.max(ymax - ymin);
    }
    (rangex,rangey)
}

pub fn compress_repeated_points_in_lines_stats(shapezs: &[ShapeZ<f64>]) -> (usize,usize){
    let mut points = 0;
    let mut repeated = 0;
    for shape in shapezs{
        if shape.points.is_empty() { continue; }
        let last = &shape.points[0];
        for p in shape.points.iter().skip(1){
            if p == last{
                repeated += 1;
            }
        }
        points += shape.points.len();
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

pub fn target_compression_type((_,rx,_,ry): Ranges) -> (u64, CompTarget){
    fn get_target(range: u64) -> CompTarget{
        if range < std::u8::MAX.into(){ CompTarget::U8 }
        else if range < std::u16::MAX.into(){ CompTarget::U16 }
        else if range < std::u32::MAX.into(){ CompTarget::U32 }
        else {CompTarget::NONE }
    }
    let max = rx.max(ry);
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

pub fn print_split_content((ps,pms,pzs,pls,plms,plzs,mps,mpms,mpzs,pgs,pgms,pgzs):
    &(VP2,VP3,VP4,VvP2,VvP3,VvP4,VvP2,VvP3,VvP4,PolysP2,PolysP3,PolysP4)){
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
