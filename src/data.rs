use bin_buffer::*;
use shapefile::*;
use shapefile::record::polygon::GenericPolygon;
use shapefile::record::polyline::GenericPolyline;
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

pub trait HasXy<T>{
    fn xy(&self) -> (T,T);
}

impl<T: Copy> HasXy<T> for &(T,T){
    fn xy(&self) -> (T,T){
        **self
    }
}

pub struct ShapeZIter<'a,T>{
    pub current: usize,
    pub shapez: &'a ShapeZ<T>,
}

impl<'a, T> ShapeZIter<'a,T>{
    pub fn from(shapez: &'a ShapeZ<T>) -> ShapeZIter<'a,T>{
        ShapeZIter{
            current: 0,
            shapez,
        }
    }
}

impl<'a, T> Iterator for ShapeZIter<'a, T>{
    type Item = &'a P2<T>;

    fn next(&mut self) -> Option<Self::Item>{
        if self.current >= self.shapez.points.len(){
            return Option::None;
        }
        let i = self.current;
        self.current += 1;
        Option::Some(&self.shapez.points[i])
    }
}

impl<'a, T> IntoIterator for &'a ShapeZ<T>{
    type Item = &'a P2<T>;
    type IntoIter = ShapeZIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter{
        ShapeZIter{
            current: 0,
            shapez: self,
        }
    }
}

pub type Polys<T> = Vec<(Vvec<T>,Vvec<T>)>;
pub type PolysP2 = Polys<P2<f64>>;
pub type PolysP3 = Polys<P3<f64>>;
pub type PolysP4 = Polys<P4<f64>>;
pub type Splitted = (VP2,VP3,VP4,VvP2,VvP3,VvP4,VvP2,VvP3,VvP4,PolysP2,PolysP3,PolysP4);

pub fn split(shapes: Vec<Shape>, logger: &mut Logger) -> Splitted{
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
    fn tp2(p: &Point) -> P2<f64> { (p.x,p.y) }
    fn tp3(p: &PointM) -> P3<f64> { (p.x,p.y,p.m) }
    fn tp4(p: &PointZ) -> P4<f64> { (p.x,p.y,p.z,p.m) }
    fn handle_polygon<PT,F,P>(pg: GenericPolygon<PT>, dst: &mut Polys<P>, conv: &F)
        where F: Fn(&PT) -> P
    {
        let mut vo: Vvec<P> = Vec::new();
        let mut vi: Vvec<P> = Vec::new();
        let convert_poly = &|s: &Vec<PT>, d: &mut Vvec<P>| { d.push(s.iter().map(|p| conv(p)).collect()); };
        pg.into_inner().iter().for_each(|x| match x {
            PolygonRing::Outer(vec) => { convert_poly(vec, &mut vo); },
            PolygonRing::Inner(vec) => { convert_poly(vec, &mut vi); },
        });
        dst.push((vo,vi));
    }
    fn convert_polyline<T,P>(pl: GenericPolyline<T>, dst: &mut Vvec<P>, cv: fn(&T) -> P) {
        pl.into_inner().iter().for_each(|x| dst.push(x.iter().map(|p| cv(p)).collect()));
    }
    fn convert_multipoint<T,P>(src: Vec<Vec<T>>, cv: fn(&T) -> P) -> Vvec<P>{
        src.iter().map(|x| x.iter().map(|p| cv(p)).collect()).collect()
    }
    for shape in shapes{
        match shape{
            Shape::NullShape => {  },
            Shape::Point(p) => { points.push(tp2(&p)); },
            Shape::PointM(p) => { pointms.push(tp3(&p)); },
            Shape::PointZ(p) => { pointzs.push(tp4(&p)); },
            Shape::Polyline(pl) => { convert_polyline(pl, &mut plines, tp2); },
            Shape::PolylineM(pl) => { convert_polyline(pl, &mut plinems, tp3); },
            Shape::PolylineZ(pl) => { convert_polyline(pl, &mut plinezs, tp4); },
            Shape::Multipoint(mp) => { mpoints.push(mp.into_inner()) },
            Shape::MultipointM(mp) => { mpointms.push(mp.into_inner()) },
            Shape::Polygon(pg) => { handle_polygon(pg, &mut polys, &tp2); },
            Shape::PolygonM(pg) => { handle_polygon(pg, &mut polyms, &tp3); },
            Shape::PolygonZ(pg) => { handle_polygon(pg, &mut polyzs, &tp4); },
            Shape::MultipointZ(mp) => { mpointzs.push(mp.into_inner()) },
            _ => {
                logger.log(Issue::UnsupportedShape);
            }
        }
    }
    let mpoints = convert_multipoint(mpoints, tp2);
    let mpointms = convert_multipoint(mpointms, tp3);
    let mpointzs = convert_multipoint(mpointzs, tp4);
    (points,pointms,pointzs,plines,plinems,plinezs,mpoints,mpointms,mpointzs,polys,polyms,polyzs)
}
