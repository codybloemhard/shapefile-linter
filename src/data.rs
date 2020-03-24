use bin_buffer::*;
use shapefile::*;
use shapefile::record::polygon::GenericPolygon;
use crate::logger::*;

pub type P2<T> = (T,T);
pub type P3<T> = (T,T,T);
pub type P4<T> = (T,T,T,T);

pub type VP2 = Vec<P2<f64>>;
pub type VP3 = Vec<P3<f64>>;
pub type VP4 = Vec<P4<f64>>;

pub type Vvec<T> = Vec<Vec<T>>;
pub type VvP2 = Vvec<P2<f64>>;
pub type VvP3 = Vvec<P3<f64>>;
pub type VvP4 = Vvec<P4<f64>>;

#[derive(Clone)]
pub struct ShapeZ<T>{
    pub points: Vec<P2<T>>,
    pub z: T,
    pub bb: (P3<T>,P3<T>),
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
        let z = if let Some(wz) = T::from_buffer(buf){ wz }
        else { return Option::None; };
        let bb0 = if let Some(wbb0) = <P3<T>>::from_buffer(buf){ wbb0 }
        else { return Option::None; };
        let bb1 = if let Some(wbb1) = <P3<T>>::from_buffer(buf){ wbb1 }
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

pub type Polys<T> = Vec<(Vvec<T>,Vvec<T>)>;
pub type PolysP2 = Polys<P2<f64>>;
pub type PolysP3 = Polys<P3<f64>>;
pub type PolysP4 = Polys<P4<f64>>;

pub fn split(shapes: Vec<Shape>, logger: &mut Logger)
    -> (VP2,VP3,VP4,VvP2,VvP3,VvP4,VvP2,VvP3,VvP4,PolysP2,PolysP3,PolysP4) {
    let mut points = Vec::new();
    let mut pointms = Vec::new();
    let mut pointzs = Vec::new();
    let mut plines = Vec::new();
    let mut plinems = Vec::new();
    let mut plinezs = Vec::new();
    let mut mpoints = Vec::new();
    let mut mpointms = Vec::new();
    let mut mpointzs = Vec::new();
    let mut polys = Vec::new();
    let mut polyms = Vec::new();
    let mut polyzs = Vec::new();
    fn pt2_to_p2(p: &Point) -> P2<f64> { (p.x,p.y) }
    fn pt3_to_p3(p: &PointM) -> P3<f64> { (p.x,p.y,p.m) }
    fn pt4_to_p4(p: &PointZ) -> P4<f64> { (p.x,p.y,p.z,p.m) }
    fn handle_polygon<PT,F,P>(pg: GenericPolygon<PT>, dst: &mut Vec<(Vvec<P>,Vvec<P>)>, conv: &F)
        where F: Fn(&PT) -> P
    {
        let mut vo: Vvec<P> = Vec::new();
        let mut vi: Vvec<P> = Vec::new();
        pg.into_inner().iter().for_each(|x| match x {
            PolygonRing::Outer(vec) => { vo.push(vec.iter().map(|p| conv(p)).collect()); },
            PolygonRing::Inner(vec) => { vi.push(vec.iter().map(|p| conv(p)).collect()); },
        });
        dst.push((vo,vi));
    }
    for shape in shapes{
        match shape{
            Shape::NullShape => {  },
            Shape::Point(p) => { points.push((p.x,p.y)); },
            Shape::PointM(p) => { pointms.push((p.x,p.y,p.m)); },
            Shape::PointZ(p) => { pointzs.push((p.x,p.y,p.z,p.m)); },
            Shape::Polyline(pl) => { pl.into_inner().iter().for_each(|x| plines.push(x.iter().map(|p| (p.x,p.y)).collect())); },
            Shape::PolylineM(pl) => { pl.into_inner().iter().for_each(|x| plinems.push(x.iter().map(|p| (p.x,p.y,p.m)).collect())); },
            Shape::PolylineZ(pl) => { pl.into_inner().iter().for_each(|x| plinezs.push(x.iter().map(|p| (p.x,p.y,p.z,p.m)).collect())); },
            Shape::Multipoint(mp) => { mpoints.push(mp.into_inner()) },
            Shape::MultipointM(mp) => { mpointms.push(mp.into_inner()) },
            Shape::Polygon(pg) => { handle_polygon(pg, &mut polys, &pt2_to_p2); },
            Shape::PolygonM(pg) => { handle_polygon(pg, &mut polyms, &pt3_to_p3); },
            Shape::PolygonZ(pg) => { handle_polygon(pg, &mut polyzs, &pt4_to_p4); },
            Shape::MultipointZ(mp) => { mpointzs.push(mp.into_inner()) },
            _ => {
                logger.log(Issue::UnsupportedShape);
            }
        }
    }
    let mpoints: VvP2 = mpoints.iter().map(|x| x.iter().map(|p| (p.x,p.y)).collect()).collect();
    let mpointms: VvP3 = mpointms.iter().map(|x| x.iter().map(|p| (p.x,p.y,p.m)).collect()).collect();
    let mpointzs: VvP4 = mpointzs.iter().map(|x| x.iter().map(|p| (p.x,p.y,p.z,p.m)).collect()).collect();
    (points,pointms,pointzs,plines,plinems,plinezs,mpoints,mpointms,mpointzs,polys,polyms,polyzs)
}
