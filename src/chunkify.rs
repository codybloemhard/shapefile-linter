use crate::data::{ShapeZ, Vvec,BB,MinMax,TTSub,HasBB,CustomShape,BoundingType,UpdateableBB};
use crate::compress::ToU64;
use crate::logger::*;

pub fn bb_in_bb_xy<T>(outer: &BB<T>, inner: &BB<T>) -> bool
    where
        T: std::cmp::PartialOrd
{
    (outer.0).0 < (inner.0).0 &&
        (outer.0).1 < (inner.0).1 &&
        (outer.1).0 > (inner.1).0 &&
        (outer.1).1 > (inner.1).1
}

pub fn get_size<T>(bb: BB<T>) -> T
    where
        T: MinMax + TTSub,
{
    let w = (bb.1).0.sub((bb.0).0);
    let h = (bb.1).1.sub((bb.0).1);
    w.min_of(h)
}

pub type Chunk<T> = (u64,u64,Vec<ShapeZ<T>>);
pub type Chunks<T> = Vec<Chunk<T>>;

pub fn cut<T>(cuts: u64, gbb: BB<T>, shapes: &[ShapeZ<T>], logger: &mut Logger) -> Chunks<T>
    where
        T: Clone + ToU64 +  BoundingType + MinMax + Copy,
{
    let mut grid: Vvec<ShapeZ<T>> = vec![vec![]; (cuts * cuts) as usize];
    let bb0x = (gbb.0).0.to();
    let bb0y = (gbb.0).1.to();
    if bb0x != 0 || bb0y != 0{
        logger.log(Issue::NonOriginBoundingbox);
        return vec![];
    }
    let bb1x = (gbb.1).0.to();
    let bb1y = (gbb.1).1.to();
    let gwid = bb1x;
    let ghei = bb1y;
    let size = gwid.max(ghei);
    let csize = (size / cuts) + 1;
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
            let mut old_cx = cx;
            let mut old_cy = cy;
            let mut points = Vec::new();
            let z = shape.z;
            for (x,y) in shape{
                let new_cx = x.to() / csize;
                let new_cy = y.to() / csize;
                if new_cx == old_cx && new_cy == old_cy{
                    points.push((*x,*y));
                }else{
                    let mut newshape = ShapeZ{
                        points,
                        z,
                        bb: T::start_box(),
                    };
                    newshape.update_bb();
                    let vpos = (old_cy * cuts + old_cx) as usize;
                    grid[vpos].push(newshape);
                    points = vec![(*x,*y)];
                    old_cx = new_cx;
                    old_cy = new_cy;
                }
            }
            let mut newshape = ShapeZ{
                points,
                z,
                bb: T::start_box(),
            };
            newshape.update_bb();
            let vpos = (old_cy * cuts + old_cx) as usize;
            grid[vpos].push(newshape);
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

pub fn pick_heights<T>(modulo: u64, chunk:Vec<ShapeZ<T>>) -> Vec<ShapeZ<T>>
    where
        T: ToU64
{
    let mut filtered = Vec::new();
    for shape in chunk{
        let z = shape.z.to();
        if z % modulo != 0{
            continue;
        }
        filtered.push(shape);
    }
    filtered
}

pub fn pick_points<T>(max: usize, mut chunk: Vec<ShapeZ<T>>) -> Vec<ShapeZ<T>>
    where
        T: Copy
{
    let mut ps = 0;
    for shape in &chunk{
        ps += shape.points_len();
    }
    let modulo = ps / max + 1;
    let mut nchunk = Vec::new();
    chunk.sort_by(|a,b| a.points_len().cmp(&b.points_len()));
    println!("shapes: {}", chunk.len());
    for shape in chunk.into_iter(){
        let points = shape.points;
        let z = shape.z;
        let bb = shape.bb;
        let mut npoints = Vec::new();
        for p in points.into_iter().step_by(modulo){
            npoints.push(p);
        }
        nchunk.push(ShapeZ{
            points: npoints,
            z,
            bb,
        });
    }
    nchunk
}
