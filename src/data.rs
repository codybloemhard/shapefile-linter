use bin_buffer::*;

pub type P2<T> = (T,T);
pub type P3 = (f64,f64,f64);

#[derive(Clone)]
pub struct ShapeZ<T>{
    pub points: Vec<P2<T>>,
    pub z: f64,
    pub bb: (P3,P3),
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
        let z = if let Some(wz) = f64::from_buffer(buf){ wz }
        else { return Option::None; };
        let bb0 = if let Some(wbb0) = <P3>::from_buffer(buf){ wbb0 }
        else { return Option::None; };
        let bb1 = if let Some(wbb1) = <P3>::from_buffer(buf){ wbb1 }
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
