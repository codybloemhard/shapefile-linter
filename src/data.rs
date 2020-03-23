use bin_buffer::*;
use shapefile::*;

pub type P2<T> = (T,T);
pub type P3<T> = (T,T,T);
pub type P4<T> = (T,T,T,T);

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

pub fn split(shapes: Vec<Shape>)
    -> (Vec<P2<f64>>,Vec<P3<f64>>,Vec<P4<f64>>,VvP2,VvP3,VvP4,VvP2,VvP3,VvP4) {
    let mut points = Vec::new();
    let mut pointms = Vec::new();
    let mut pointzs = Vec::new();
    let mut plines = Vec::new();
    let mut plinems = Vec::new();
    let mut plinezs = Vec::new();
    let mut mpoints = Vec::new();
    let mut mpointms = Vec::new();
    let mut mpointzs = Vec::new();
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
            Shape::MultipointZ(mp) => { mpointzs.push(mp.into_inner()) },
            _ => {  }
        }
    }
    let mpoints: VvP2 = mpoints.iter().map(|x| x.iter().map(|p| (p.x,p.y)).collect()).collect();
    let mpointms: VvP3 = mpointms.iter().map(|x| x.iter().map(|p| (p.x,p.y,p.m)).collect()).collect();
    let mpointzs: VvP4 = mpointzs.iter().map(|x| x.iter().map(|p| (p.x,p.y,p.z,p.m)).collect()).collect();
    (points,pointms,pointzs,plines,plinems,plinezs,mpoints,mpointms,mpointzs)
}
