use crate::data::{ShapeZ, Vvec,BB,MinMax,HasBB,CustomShape,BoundingType,StretchableBB};
use crate::logger::*;
use crate::compress::FromU64;
use std::ops::{Div,Add,Mul};

pub fn bb_in_bb_xy<T>(outer: &BB<T>, inner: &BB<T>) -> bool
    where
        T: std::cmp::PartialOrd
{
    (outer.0).0 <= (inner.0).0 &&
        (outer.0).1 <= (inner.0).1 &&
        (outer.1).0 >= (inner.1).0 &&
        (outer.1).1 >= (inner.1).1
}

pub type Chunk<T> = (u64,u64,Vec<ShapeZ<T>>);
pub type Chunks<T> = Vec<Chunk<T>>;

pub fn cut<T>(cuts: u64, gbb: BB<T>, shapes: &[ShapeZ<T>], logger: &mut Logger) -> Chunks<T>
    where
        T: Clone + Copy + Default + PartialEq + PartialOrd + Into<usize>,
        T: FromU64 +  BoundingType + MinMax,
        T: Div<Output = T> + Add<Output = T> + Mul<Output = T>,
{
    let cuts_u64 = cuts;
    let cuts = T::from(cuts);
    let mut grid: Vvec<ShapeZ<T>> = vec![vec![]; (cuts_u64 * cuts_u64) as usize];
    let bb0x = (gbb.0).0;
    let bb0y = (gbb.0).1;
    if bb0x != T::default() || bb0y != T::default(){
        logger.log(Issue::NonOriginBoundingbox);
        return vec![];
    }
    let gwid = (gbb.1).0;
    let ghei = (gbb.1).1;
    let csizex = (gwid / cuts) + T::from(1u64);
    let csizey = (ghei / cuts) + T::from(1u64);
    for shape in shapes{
        let bb = shape.bounding_box();
        let x0 = (bb.0).0;
        let y0 = (bb.0).1;
        let x1 = (bb.1).0;
        let y1 = (bb.1).1;
        let cx = x0 / csizex;
        let cy = y0 / csizey;
        let sbb = ((x0,y0,T::default()),(x1,y1,T::default()));
        let cbb = ((cx * csizex, cy * csizey, T::default()),
                    (cx * csizex + csizex, cy * csizey + csizey,T::default()));
        let outside = bb_out_bb_xy(&cbb, &sbb);
        if outside { continue; }
        let inside = bb_in_bb_xy(&cbb, &sbb);
        if inside{
            let vpos = (cy * cuts + cx).into();
            grid[vpos].push(shape.clone());
        }else if !shape.points.is_empty(){
            let (fx,fy) = &shape.points[0];
            let mut old_cx = (*fx) / csizex;
            let mut old_cy = (*fy) / csizey;
            let mut points = Vec::new();
            let mut lastx = T::default();
            let mut lasty = T::default();
            let z = shape.z;
            for (x,y) in shape{
                let new_cx = (*x) / csizex;
                let new_cy = (*y) / csizey;
                if new_cx == old_cx && new_cy == old_cy{
                    points.push((*x,*y));
                }else{
                    points.push((*x,*y));
                    let mut newshape = ShapeZ{
                        points,
                        z,
                        bb: T::start_box(),
                    };
                    newshape.stretch_bb();
                    let vpos: usize = (old_cy * cuts + old_cx).into();
                    grid[vpos].push(newshape);
                    points = vec![(lastx,lasty),(*x,*y)];
                    old_cx = new_cx;
                    old_cy = new_cy;
                }
                lastx = *x;
                lasty = *y;
            }
            let mut newshape = ShapeZ{
                points,
                z,
                bb: T::start_box(),
            };
            newshape.stretch_bb();
            let vpos = (old_cy * cuts + old_cx).into();
            grid[vpos].push(newshape);
        }
    }
    let mut chunks = Vec::new();
    for (i,vec) in grid.into_iter().enumerate(){
        let i = i as u64;
        let x = i % cuts_u64;
        let y = i / cuts_u64;
        chunks.push((x,y,vec));
    }
    chunks
}

pub fn bb_out_bb_xy<T>(outer: &BB<T>, inner: &BB<T>) -> bool
    where
        T: std::cmp::PartialOrd
{
    ((outer.0).0 > (inner.1).0 || (outer.1).0 < (inner.0).0) &&
    ((outer.0).1 > (inner.1).1 || (outer.1).1 < (inner.0).1)
}

pub fn pick_heights<T>(modulo: u64, chunk:Vec<ShapeZ<T>>) -> Vec<ShapeZ<T>>
    where
        T: Into<u64> + Copy
{
    let mut filtered = Vec::new();
    for shape in chunk{
        let z = shape.z.into();
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
    println!("shapes: {}, mod: {}", chunk.len(), modulo);
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
