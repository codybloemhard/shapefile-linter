use crate::data::ShapeZ;
use crate::data::Vvec;
use crate::data::BB;
use crate::data::HasXy;
use crate::data::MinMax;
use crate::data::TTSub;
use crate::data::HasBB;
use crate::compress::ToU64;
use std::convert::TryInto;
use crate::logger::Logger;
use crate::logger::*;

pub fn bb_in_bb_xy<T>(outer: &BB<T>, inner: &BB<T>) -> bool
    where
        T: std::cmp::PartialOrd
{
    if (outer.0).0 < (inner.0).0 &&
        (outer.0).1 < (inner.0).1 &&
        (outer.1).0 > (inner.1).0 &&
        (outer.1).1 > (inner.1).1
        { true }
    else
        { false }
}

pub fn get_size<T>(bb: BB<T>) -> T
    where
        T: MinMax + TTSub,
{
    let w = (bb.1).0.sub((bb.0).0);
    let h = (bb.1).1.sub((bb.0).1);
    w.min_of(h)
}

pub type Chunks<T> = Vec<(u64,u64,Vec<ShapeZ<T>>)>;

pub fn cut<T>(cuts: u64, gbb: BB<T>, shapes: &Vec<ShapeZ<T>>, logger: &mut Logger) -> Chunks<T>
    where
        T: Clone + ToU64
{
    let mut grid: Vvec<ShapeZ<T>> = vec![vec![]; (cuts * cuts) as usize];
    let bb0x = (gbb.0).0.to();
    let bb0y = (gbb.0).1.to();
    let bb1x = (gbb.1).0.to();
    let bb1y = (gbb.1).1.to();
    let gwid = bb1x - bb0x;
    let ghei = bb1y - bb0y;
    let size = gwid.max(ghei);
    let csize = size / cuts;
    for shape in shapes{
        let bb = shape.bounding_box();
        let x0 = (bb.0).0.to();
        let y0 = (bb.0).1.to();
        let x1 = (bb.1).0.to();
        let y1 = (bb.1).1.to();
        let cx = x0 / csize;
        let cy = y0 / csize;
        let sbb = ((x0,y0,0),(x1,y1,0));
        let cbb = ((cx * csize, cy * csize, 0),(cx * csize + csize, cy * csize + csize,0));
        let inside = bb_in_bb_xy(&cbb, &sbb);
        if !inside {
            logger.log(Issue::MultiChunkShape);
            continue;
        }
        let vpos = (cy * cuts + cx) as usize;
        grid[vpos].push(shape.clone());
    }
    let mut chunks = Vec::new();
    for (i,vec) in grid.into_iter().enumerate(){
        let i = i as u64;
        let x = i % cuts;
        let y = i / cuts;
        chunks.push((x,y,vec));
    }
    chunks
}
