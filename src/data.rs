use std::borrow::Borrow;
use bin_buffer::*;
use shapefile::*;
use shapefile::record::polygon::GenericPolygon;
use shapefile::record::polyline::GenericPolyline;
use crate::logger::*;
use crate::triangulate::PolyTriangle;

pub type P2<T> = (T,T);
pub type P3<T> = (T,T,T);
pub type P4<T> = (T,T,T,T);
pub type BB<T> = (P3<T>,P3<T>);

pub type VP2 = Vec<P2<f64>>;
pub type VP3 = Vec<P3<f64>>;
pub type VP4 = Vec<P4<f64>>;

pub type Vvec<T> = Vec<Vec<T>>;
pub type VvP2 = Vvec<P2<f64>>;
pub type VvP3 = Vvec<P3<f64>>;
pub type VvP4 = Vvec<P4<f64>>;
// Be sure it has at least 2 components
pub trait HasXy<T>{
    fn xy(&self) -> (T,T);
}

impl<T: Copy + Default> HasXy<T> for &(T,T){
    fn xy(&self) -> (T,T){
        **self
    }
}

impl <T: Copy + Default> HasXy<T> for &(T,T,T){
    fn xy(&self) -> (T,T){
        (self.0,self.1)
    }
}
// Be sure it has at least three components
pub trait HasXyz<T>{
    fn xyz(&self) -> (T,T,T);
}

impl<T: Copy + Default> HasXyz<T> for &(T,T){
    fn xyz(&self) -> (T,T,T){
        (self.0,self.1,T::default())
    }
}

impl<T: Copy> HasXyz<T> for &(T,T,T){
    fn xyz(&self) -> (T,T,T){
        **self
    }
}
// Be able to perform min,max, and have min and max values.
// This is because float does not implement Cmp, so i can't use normal min and max.
pub trait MinMax{
    fn minv() -> Self;
    fn maxv() -> Self;
    fn min_of(self, x: Self) -> Self;
    fn max_of(self, x: Self) -> Self;
}
// Macro that can implement MinMax for given type
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
// Actual implementations
ImplMinMax!(f64);ImplMinMax!(f32);ImplMinMax!(u64);ImplMinMax!(u32);ImplMinMax!(u16);ImplMinMax!(u8);
// These types can stretch a bounding box
pub trait Bounded<T>{
    fn stretch_bound(self, bb: &mut BB<T>);
}
// Any 2d value that has MinMax and Copy you can stretch a bound of your own inner type
impl<T: MinMax + Copy> Bounded<T> for &(T,T){
    fn stretch_bound(self, bb: &mut BB<T>){
        (bb.0).0 = (bb.0).0.min_of(self.0);
        (bb.0).1 = (bb.0).1.min_of(self.1);
        (bb.1).0 = (bb.1).0.max_of(self.0);
        (bb.1).1 = (bb.1).1.max_of(self.1);
    }
}
// Same for 3d values but they also effect the z(obv)
impl<T: MinMax + Copy> Bounded<T> for &(T,T,T){
    fn stretch_bound(self, bb: &mut BB<T>){
        (bb.0).0 = (bb.0).0.min_of(self.0);
        (bb.0).1 = (bb.0).1.min_of(self.1);
        (bb.0).2 = (bb.0).2.min_of(self.2);
        (bb.1).0 = (bb.1).0.max_of(self.0);
        (bb.1).1 = (bb.1).1.max_of(self.1);
        (bb.1).2 = (bb.1).2.max_of(self.2);
    }
}
// Bounding boxes can consist of these types
pub trait BoundingType{
    // Default bounding box(0)
    fn default_box() -> BB<Self> where Self: Sized;
    // Bounding box that can be stretched
    // You need to stretch it at least one time
    // Otherwise it is an invalid box
    // Since min will be bigger than max
    fn start_box() -> BB<Self> where Self: Sized;
}
// Copy, Default, MinMax, Sized, are enough to make sure it can be a BoundingType
impl<T> BoundingType for T
    where
        T: Copy + Default + MinMax + Sized
{
    fn default_box() -> BB<T>{
        ((T::default(),T::default(),T::default()),
        (T::default(),T::default(),T::default()))
    }

    fn start_box() -> BB<T>{
        ((T::maxv(),T::maxv(),T::maxv()),
        (T::minv(),T::minv(),T::minv()))
    }
}
// Every custom type of shape has points so we can know how many points it has
pub trait CustomShape{
    fn points_len(&self) -> usize;
}
// Indicates that a collection has a boundingbox
pub trait HasBB<T>{
    // Return the boundingbox
    fn bounding_box(&self) -> &BB<T>;
    // Set the boundingbox
    fn set_bounding_box(&mut self, bb: BB<T>);
}
// Indicates that it has a stretchable boundingbox
pub trait StretchableBB{
    // Stretch the box using it's own points
    fn stretch_bb(&mut self);
}
// Macro that implements StretchableBB for us
// Assumes that it has HasBB
macro_rules! ImplStretchableBB{
    ($ttype:ident) => {
        impl<T> StretchableBB for $ttype<T>
            where
                T: BoundingType + MinMax + Copy,
        {
            fn stretch_bb(&mut self){
                if self.points_len() == 0 { return; }
                let mut bb = T::start_box();
                let b: &$ttype<T> = self.borrow();
                for x in b{
                    x.stretch_bound(&mut bb);
                }
                self.set_bounding_box(bb);
            }
        }
    }
}
// Implement for collections
ImplStretchableBB!(ShapeZ);
ImplStretchableBB!(PolygonZ);
ImplStretchableBB!(PolyTriangle);
ImplStretchableBB!(StyledLine);
// When a shape can update it's own boundingbox, to be consistent with it's points afterwards
pub trait UpdateableBB{
    fn update_bb(&mut self);
}
// ShapeZ kan update it's boundingbox
impl<T> UpdateableBB for ShapeZ<T>
    where
        T: BoundingType + MinMax + Copy,
{
    fn update_bb(&mut self){
        let mut bb = *self.bounding_box();
        (bb.0).2 = (bb.0).2.min_of(self.z);
        (bb.1).2 = (bb.1).2.max_of(self.z);
        self.set_bounding_box(bb);
    }
}
// These ones do not use UpdateableBB but we need them for generics
impl<T> UpdateableBB for PolygonZ<T>
    where
    T: BoundingType + MinMax + Copy,
{ fn update_bb(&mut self){ /* noop */ } }
impl<T> UpdateableBB for PolyTriangle<T>
    where
    T: BoundingType + MinMax + Copy,
{ fn update_bb(&mut self){ /* noop */ } }
impl<T> UpdateableBB for StyledLine<T>
{ fn update_bb(&mut self){ /* noop */ } }
// Stretch a bounding box with other boundingboxes
// This to get the global boundingbox
pub fn get_global_bb<T,U>(shapes: &[U]) -> BB<T>
    where
        U: HasBB<T>,
        T: MinMax + Copy + BoundingType,
{
    if shapes.is_empty() {
        return T::default_box();
    }
    let mut minx = T::maxv();
    let mut maxx = T::minv();
    let mut miny = T::maxv();
    let mut maxy = T::minv();
    let mut minz = T::maxv();
    let mut maxz = T::minv();
    for shape in shapes{
        let bb = shape.bounding_box();
        minx = minx.min_of((bb.0).0);
        miny = miny.min_of((bb.0).1);
        minz = minz.min_of((bb.0).2);
        maxx = maxx.max_of((bb.1).0);
        maxy = maxy.max_of((bb.1).1);
        maxz = maxz.max_of((bb.1).2);
    }
    ((minx,miny,minz),(maxx,maxy,maxz))
}
// ShapeZ: a height line, all it's points are on the same height
#[derive(Clone)]
pub struct ShapeZ<T>{
    pub points: Vec<P2<T>>,
    pub z: T,
    pub bb: BB<T>,
}
// We want to export and import ShapeZ as buffer
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
        let z = T::from_buffer(buf)?;
        let bb0 = <P3<T>>::from_buffer(buf)?;
        let bb1 = <P3<T>>::from_buffer(buf)?;
        let len = u64::from_buffer(buf)?;
        let mut vec = Vec::new();
        for _ in 0..len{
            let p = <P2<T>>::from_buffer(buf)?;
            vec.push(p);
        }
        Option::Some(Self{
            points: vec,
            z,
            bb: (bb0,bb1),
        })
    }
}
// The next two are trivial
impl<T> CustomShape for ShapeZ<T>{
    fn points_len(&self) -> usize{
        self.points.len()
    }
}

impl<T> HasBB<T> for ShapeZ<T>{
    fn bounding_box(&self) -> &BB<T>{
        &self.bb
    }

    fn set_bounding_box(&mut self, bb: BB<T>){
        self.bb = bb
    }
}
// An iterator state that only has a reference
pub struct ShapeZIter<'a,T>{
    pub current: usize,
    pub shapez: &'a ShapeZ<T>,
}
// ShapeZIter is an Iterator, it yields references to the points
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
// You can take an iterator from &ShapeZ
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
// Styles line is a line with a style
#[derive(Clone)]
pub struct StyledLine<T>{
    pub points: Vec<P2<T>>,
    pub style: usize,
    pub bb: BB<T>,
}
// Make Styled Lines from raw data but cast to int. We need this as some of the generics wont play
// nice with floats.
impl<T> StyledLine<T>{
    pub fn from_as_int((style,ps): (usize, Vvec<P2<f64>>), col: &mut Vec<StyledLine<u32>>) {
        for l in ps.into_iter().map(|v| v.into_iter().map(|(x,y)| (x as u32, y as u32)).collect::<Vec<_>>()){
            let mut temp = StyledLine::<u32>{
                points: l,
                style,
                bb: u32::start_box(),
            };
            temp.stretch_bb();
            col.push(temp);
        }
    }
}
// We want to export and import StyledLine as buffer
impl<T: Bufferable + Clone> Bufferable for StyledLine<T>{
    fn into_buffer(self, buf: &mut Buffer){
        self.style.into_buffer(buf);
        self.bb.0.into_buffer(buf);
        self.bb.1.into_buffer(buf);
        self.points.into_buffer(buf);
    }

    fn copy_into_buffer(&self, buf: &mut Buffer){
        self.clone().into_buffer(buf);
    }

    fn from_buffer(buf: &mut ReadBuffer) -> Option<Self>{
        let style = usize::from_buffer(buf)?;
        let bb0 = <P3<T>>::from_buffer(buf)?;
        let bb1 = <P3<T>>::from_buffer(buf)?;
        let vec = Vec::<P2<T>>::from_buffer(buf)?;
        Option::Some(Self{
            points: vec,
            style,
            bb: (bb0,bb1),
        })
    }
}
// The next two are trivial
impl<T> CustomShape for StyledLine<T>{
    fn points_len(&self) -> usize{
        self.points.len()
    }
}

impl<T> HasBB<T> for StyledLine<T>{
    fn bounding_box(&self) -> &BB<T>{
        &self.bb
    }

    fn set_bounding_box(&mut self, bb: BB<T>){
        self.bb = bb
    }
}
// Again we need a state to iterate over the collection
pub struct StyledLineIter<'a,T>{
    pub current: usize,
    pub sline: &'a StyledLine<T>,
}
// Standard iterator. Multiple of these boys have iterators like this and it should be refactored.
// Something like that it has a point collection and that one yield an iterator and only define
// iterators for the point collections, removing doubly implemented iterators.
impl<'a, T> Iterator for StyledLineIter<'a, T>{
    type Item = &'a P2<T>;
    fn next(&mut self) -> Option<Self::Item>{
        let cur = self.current;
        if cur >= self.sline.points.len() {
            return Option::None;
        }
        self.current = cur + 1;
        Option::Some(&self.sline.points[cur])
    }
}
// Make possible to get the iterator
impl<'a, T> IntoIterator for &'a StyledLine<T>{
    type Item = &'a P2<T>;
    type IntoIter = StyledLineIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter{
        StyledLineIter{
            current: 0,
            sline: self,
        }
    }
}
// Struct for PolygonZ's
// Has inner and outer rings, in no order
#[derive(Clone)]
pub struct PolygonZ<T>{
    pub inners: Vvec<P3<T>>,
    pub outers: Vvec<P3<T>>,
    pub bb: BB<T>,
    pub style: usize,
}
// Just a function that we use to build this struct from the raw unpacked data
// we get from the shapfile
impl<T: BoundingType + Copy> PolygonZ<T>{
    pub fn from(raw: Poly<P4<T>>, style: usize) -> Self{
        fn crunch<T>(raw: Vvec<P4<T>>) -> Vvec<P3<T>>{
            let mut col = Vec::new();
            for outer in raw{
                let mut vec = Vec::new();
                for (x,y,z,_) in outer{
                    vec.push((x,y,z));
                }
                col.push(vec);
            }
            col
        }
        Self{
            outers: crunch(raw.0),
            inners: crunch(raw.1),
            bb: T::start_box(),
            style,
        }
    }
}
// Not all generic functions like floats so we have a cast function that transfroms it into a int
// collection. This is kind of ugly and would like some refactoring.
pub fn int_cast(pzf64: PolygonZ<f64>) -> PolygonZ::<u32>
{
    let cast = |(x,y,z)| (x as u32, y as u32, z as u32);
    let ((a,b,c),(d,e,f)) = pzf64.bb;
    PolygonZ::<u32>{
        inners: pzf64.inners.into_iter().map(|v| v.into_iter().map(cast).collect::<Vec<_>>()).collect::<Vec<_>>(),
        outers: pzf64.outers.into_iter().map(|v| v.into_iter().map(cast).collect::<Vec<_>>()).collect::<Vec<_>>(),
        bb: ((a as u32, b as u32, c as u32),(d as u32, e as u32, f as u32)),
        style: pzf64.style,
    }
}
// Next 3 implementations are trivial
impl<T> CustomShape for PolygonZ<T>{
    fn points_len(&self) -> usize{
        self.inners.len() + self.outers.len()
    }
}

impl<T> HasBB<T> for PolygonZ<T>{
    fn bounding_box(&self) -> &BB<T>{
        &self.bb
    }

    fn set_bounding_box(&mut self, bb: BB<T>){
        self.bb = bb
    }
}

impl<T: Bufferable + Clone> Bufferable for PolygonZ<T>{
    fn into_buffer(self, buf: &mut Buffer){
        self.bb.0.into_buffer(buf);
        self.bb.1.into_buffer(buf);
        self.outers.into_buffer(buf);
        self.inners.into_buffer(buf);
        self.style.into_buffer(buf);
    }

    fn copy_into_buffer(&self, buf: &mut Buffer){
        self.clone().into_buffer(buf);
    }

    fn from_buffer(buf: &mut ReadBuffer) -> Option<Self>{
        let bb0 = <P3<T>>::from_buffer(buf)?;
        let bb1 = <P3<T>>::from_buffer(buf)?;
        let outers = Vvec::<P3<T>>::from_buffer(buf)?;
        let inners = Vvec::<P3<T>>::from_buffer(buf)?;
        let style = usize::from_buffer(buf)?;
        Option::Some(Self{
            outers,
            inners,
            bb: (bb0,bb1),
            style,
        })
    }
}
// Again we need a state to iterate over the collection
// This one is more complicated
// We have to iterate over many nested vectors
// As if it was one long one
pub struct PolygonZIter<'a,T>{
    pub current: usize,
    pub outer: bool,
    pub index: usize,
    pub poly: &'a PolygonZ<T>,
}
// Here we define how we actually loop over seperate nested vectors
impl<'a, T> Iterator for PolygonZIter<'a, T>{
    type Item = &'a P3<T>;
    //Note: this looks very clunky but it this way because the &mut gets in the way of the & if the
    //closure captures self
    fn next(&mut self) -> Option<Self::Item>{
        let iter_sub = |sub: &'a Vvec<P3<T>>, mut ind: usize, mut cur: usize|{
            loop{
                if ind >= sub.len(){
                    return (Option::None,ind,cur);
                }
                if cur >= sub[ind].len(){
                    ind += 1;
                    cur = 0;
                }else{ break; }
            }
            (Option::Some(&sub[ind][cur]),ind,cur + 1)
        };
        let mut ind = self.index;
        let mut cur = self.current;
        let is_inner = !self.outer;
        if is_inner {
            let (r,i,c) = iter_sub(&self.poly.inners,ind,cur);
            self.index = i;
            self.current = c;
            return r;
        }
        let (r,i,c) = iter_sub(&self.poly.outers, ind, cur);
        ind = i;
        cur = c;
        if r.is_some(){
            self.index = i;
            self.current = c;
            r
        }
        else{
            self.outer = false;
            let (r,i,c) = iter_sub(&self.poly.inners, ind, cur);
            self.index = i;
            self.current = c;
            r
        }
    }
}
// And ofcouse we can turn &PolygonZ into an iterator that yields &P3
impl<'a, T> IntoIterator for &'a PolygonZ<T>{
    type Item = &'a P3<T>;
    type IntoIter = PolygonZIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter{
        PolygonZIter{
            current: 0,
            outer: true,
            index: 0,
            poly: self,
        }
    }
}
// Same simple implementations for PolyTriangle.
impl<T> CustomShape for PolyTriangle<T>{
    fn points_len(&self) -> usize{
        self.vertices.len()
    }
}
impl<T> HasBB<T> for PolyTriangle<T>{
    fn bounding_box(&self) -> &BB<T>{
        &self.bb
    }

    fn set_bounding_box(&mut self, bb: BB<T>){
        self.bb = bb
    }
}
// Again we need a state to iterate over the collection
pub struct PolyTriangleIter<'a,T>{
    pub current: usize,
    pub poly: &'a PolyTriangle<T>,
}
impl<'a, T> Iterator for PolyTriangleIter<'a, T>{
    type Item = &'a P2<T>;
    fn next(&mut self) -> Option<Self::Item>{
        if self.current >= self.poly.vertices.len(){
            return Option::None;
        }
        let i = self.current;
        self.current += 1;
        Option::Some(&self.poly.vertices[i])
    }
}
impl<'a, T> IntoIterator for &'a PolyTriangle<T>{
    type Item = &'a P2<T>;
    type IntoIter = PolyTriangleIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter{
        PolyTriangleIter{
            current: 0,
            poly: self,
        }
    }
}
// Thank the gods for recursive generic type definitions
pub type Poly<T> = (Vvec<T>,Vvec<T>);
pub type Polys<T> = Vec<Poly<T>>;
pub type PolysP2 = Polys<P2<f64>>;
pub type PolysP3 = Polys<P3<f64>>;
pub type PolysP4 = Polys<P4<f64>>;
pub type Splitted = (VP2,VP3,VP4,VvP2,VvP3,VvP4,VvP2,VvP3,VvP4,PolysP2,PolysP3,PolysP4);
// Take the shapefile and turn it into seperate collections
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
    // wow, much map, much iter, much collect
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

