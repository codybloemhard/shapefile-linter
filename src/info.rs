use super::data::ShapeZ;

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
