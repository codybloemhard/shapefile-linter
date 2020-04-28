use crate::data::{ShapeZ, Vvec,BB,MinMax,HasBB,CustomShape,BoundingType,StretchableBB};
use crate::logger::*;
use crate::compress::FromU64;
use std::ops::{Div,Add,Mul};
use std::borrow::Borrow;

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
        T: Copy + Eq
{
    let mut ps = 0;
    for shape in &chunk{
        ps += shape.points_len();
    }
    let modulo = ps / max + 1;
    let mut nchunk = Vec::new();
    chunk.sort_by(|a,b| a.points_len().cmp(&b.points_len()));
    // println!("shapes: {}, mod: {}", chunk.len(), modulo);
    for shape in chunk.into_iter(){
        let points = shape.points;
        let z = shape.z;
        let bb = shape.bb;
        let mut npoints = Vec::new();
        let last = points.len() - 1;
        for (i,p) in points.into_iter().enumerate(){
            if i % modulo == 0 || i == last{
                npoints.push(p);
            }
        }
        nchunk.push(ShapeZ{
            points: npoints,
            z,
            bb,
        });
    }
    nchunk
}

pub fn optimize_lines<T>(mut old: Vec<ShapeZ<T>>) -> Vec<ShapeZ<T>>
    where
        T: Copy + Eq + Default + MinMax,
{
    type Fl<T> = ((T,T),(T,T));
    enum Fres { FF, FL, LF, LL };

    fn get_fl<T: Copy>(shape: &ShapeZ<T>) -> Fl<T>{
        let first = shape.points[0];
        let last = shape.points[shape.points.len() - 1];
        (first,last)
    }

    fn find_other<T>(shape: &ShapeZ<T>, others: &mut Vec<ShapeZ<T>>) -> Option<(Fres,ShapeZ<T>)>
        where
            T: PartialEq + Copy,
    {
        let (f0,l0) = get_fl(shape);
        let mut fres = Fres::FF;
        let mut ind = std::usize::MAX;
        let b: &Vec<ShapeZ<T>> = others.borrow();
        for (i,other) in b.iter().enumerate(){
            if shape.z != other.z { continue; }
            let (f1,l1) = get_fl(other);
            if f0 == f1 { fres = Fres::FF; ind = i; break; }
            if f0 == l1 { fres = Fres::FL; ind = i; break; }
            if l0 == f1 { fres = Fres::LF; ind = i; break; }
            if l0 == l1 { fres = Fres::LL; ind = i; break; }
        }
        if ind == std::usize::MAX{
            None
        }else{
            let shape = others.swap_remove(ind);
            Some((fres,shape))
        }
    }

    fn reversed<T>(mut v: Vec<T>) -> Vec<T>{
        v.reverse();
        v
    }

    fn conc<T>(mut a: Vec<T>, b: Vec<T>) -> Vec<T>{
        a.extend(b);
        a
    }

    fn merge<T>(fres: Fres, shape: ShapeZ<T>, oshape: ShapeZ<T>) -> ShapeZ<T>
        where
            T: BoundingType + MinMax + Copy,
    {
        let sp = shape.points;
        let op = oshape.points;
        let z = shape.z;
        let np = match fres{
            Fres::FF => conc(reversed(sp),op),
            Fres::FL => conc(op,sp),
            Fres::LF => conc(sp,op),
            Fres::LL => conc(sp,reversed(op)),
        };
        let mut nz = ShapeZ{
            points: np,
            z,
            bb: T::start_box(),
        };
        nz.stretch_bb();
        nz
    }

    let mut independents = Vec::new();
    while !old.is_empty(){
        let shape = old.pop().unwrap();
        let other = find_other(&shape, &mut old);
        if let Some((fres,oshape)) = other{
            let news = merge(fres,shape,oshape);
            old.push(news);
        }else{
            independents.push(shape);
        }
    }
    independents
}
