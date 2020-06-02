use crate::data::{ShapeZ, Vvec,BB,MinMax,HasBB,CustomShape,BoundingType,StretchableBB};
use crate::logger::*;
use crate::triangulate::PolyTriangle;
use std::ops::{Div,Add,Mul,Sub};
use std::borrow::Borrow;
use std::collections::HashMap;
use ass::*;
// Returns true if inner is completely inside outer, else false
pub fn bb_in_bb_xy<T>(outer: &BB<T>, inner: &BB<T>) -> bool
    where
        T: std::cmp::PartialOrd
{
    (outer.0).0 <= (inner.0).0 &&
        (outer.0).1 <= (inner.0).1 &&
        (outer.1).0 >= (inner.1).0 &&
        (outer.1).1 >= (inner.1).1
}

pub type ChunkLine<T> = (u64,u64,Vec<ShapeZ<T>>);
pub type ChunksLine<T> = Vec<ChunkLine<T>>;
// take shapes, a global boundingbox and the amount of cuts to do over each axis
// cuts the shapes into a scales regular grid
pub fn cut<T>(cuts: u64, gbb: BB<T>, shapes: &[ShapeZ<T>], logger: &mut Logger) -> ChunksLine<T>
    where
        T: Clone + Copy + Default + PartialEq + PartialOrd + Into<usize>,
        T: BoundingType + MinMax,
        T: Div<Output = T> + Add<Output = T> + Mul<Output = T>,
        u64: Ass<T>,
{
    let cuts_u64 = cuts;
    let cuts = (cuts).ass();
    let mut grid: Vvec<ShapeZ<T>> = vec![vec![]; (cuts_u64 * cuts_u64) as usize];
    let bb0x = (gbb.0).0;
    let bb0y = (gbb.0).1;
    if bb0x != T::default() || bb0y != T::default(){
        logger.log(Issue::NonOriginBoundingbox);
        return vec![];
    }
    let gwid = (gbb.1).0;
    let ghei = (gbb.1).1;
    let csizex = (gwid / cuts) + (1u64).ass();
    let csizey = (ghei / cuts) + (1u64).ass();
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
        if inside{ // if the whole shape is in the chunk, just put it in
            let vpos = (cy * cuts + cx).into();
            grid[vpos].push(shape.clone());
        }else if !shape.points.is_empty(){ // we need to cut the shape
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
                }else{ // if we changed chunk ...
                    points.push((*x,*y)); // keep out of chunk point to keep the line partly inside the chunk
                    let mut newshape = ShapeZ{
                        points,
                        z,
                        bb: T::start_box(),
                    };
                    newshape.stretch_bb(); // calculate the right boundingbox
                    let vpos: usize = (old_cy * cuts + old_cx).into();
                    grid[vpos].push(newshape);
                    points = vec![(lastx,lasty),(*x,*y)]; // start a new collection for the new chunk
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
            }; // round up and push the last chunk
            newshape.stretch_bb();
            let vpos = (old_cy * cuts + old_cx).into();
            grid[vpos].push(newshape);
        }
    }
    let mut chunks = Vec::new(); // just assign chunk numbers
    for (i,vec) in grid.into_iter().enumerate(){
        let i = i as u64;
        let x = i % cuts_u64;
        let y = i / cuts_u64;
        chunks.push((x,y,vec));
    }
    chunks
}
// true if inner is completely outside outer, else false
pub fn bb_out_bb_xy<T>(outer: &BB<T>, inner: &BB<T>) -> bool
    where
        T: std::cmp::PartialOrd
{
    ((outer.0).0 > (inner.1).0 || (outer.1).0 < (inner.0).0) &&
    ((outer.0).1 > (inner.1).1 || (outer.1).1 < (inner.0).1)
}
// remove heightlines that are not dividable by the modulo
// this to have less points and make the map more readable
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
// Simplify the lines by just taking out every n point
pub fn pick_points<T>(max: usize, chunk: Vec<ShapeZ<T>>) -> Vec<ShapeZ<T>>
    where
        T: Copy + Eq
{
    let mut ps = 0;
    for shape in &chunk{
        ps += shape.points_len();
    }
    let modulo = ps / max + 1;
    let mut nchunk = Vec::new();
    for shape in chunk.into_iter(){
        let points = shape.points;
        let z = shape.z;
        let bb = shape.bb;
        let mut npoints = Vec::new();
        let last = points.len() - 1;
        for (i,p) in points.into_iter().enumerate(){
            if i % modulo == 0 || i == last{ // never leave out last or first point
                npoints.push(p); // because the lines will not fit together that way
            } // and you will lose the opportunity to do "optimize_lines"
        }
        nchunk.push(ShapeZ{
            points: npoints,
            z,
            bb,
        });
    }
    nchunk
}
// Merge lines with the same start or end point
// Will result in less lines and less points
// Will increase drawing performance if you have lots of lines with not so much points
// As you will have in low LOD chunks
pub fn optimize_lines<T>(mut old: Vec<ShapeZ<T>>) -> Vec<ShapeZ<T>>
    where
        T: Copy + Eq + Default + MinMax,
{
    type Fl<T> = ((T,T),(T,T));
    enum Fres { FF, FL, LF, LL };
    // just get first and last points
    fn get_fl<T: Copy>(shape: &ShapeZ<T>) -> Fl<T>{
        let first = shape.points[0];
        let last = shape.points[shape.points.len() - 1];
        (first,last)
    }
    // Return the first shape that connects to the input shape, or None
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
            let shape = others.swap_remove(ind); // O(1) remove, will not preserve order, we don't care
            Some((fres,shape))
        }
    }
    // return reversed vec
    fn reversed<T>(mut v: Vec<T>) -> Vec<T>{
        v.reverse();
        v
    }
    // return concatened vec
    // we assume last a = first b
    // and they are both non empty
    fn conc<T>(mut a: Vec<T>, b: Vec<T>) -> Vec<T>{
        a.pop(); // remove the repeated point
        a.extend(b);
        a
    }
    // merge two shapes into one, knowing that they connect
    fn merge<T>(fres: Fres, shape: ShapeZ<T>, oshape: ShapeZ<T>) -> ShapeZ<T>
        where
            T: BoundingType + MinMax + Copy,
    {
        let sp = shape.points;
        let op = oshape.points;
        let z = shape.z;
        let np = match fres{ // note that reversing a line does not change how it looks
            Fres::FF => conc(reversed(sp),op), // reverse sp so that last sp = first op, we concat
            Fres::FL => conc(op,sp), // begin sp = end op so just put sp after op
            Fres::LF => conc(sp,op), // exactly other way around
            Fres::LL => conc(sp,reversed(op)), // just the inverse of the first case
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
    loop {
        let shape = if let Some(s) = old.pop() { s }
        else { break; };
        let other = find_other(&shape, &mut old);
        if let Some((fres,oshape)) = other{ // if we find a match, merge and put it back
            let news = merge(fres,shape,oshape);
            old.push(news);
        }else{
            independents.push(shape); // if not, put it in the new vec
        }
    }
    independents
}

fn xy_to_chunk<T>((x,y): (T,T), (csizex,csizey,offx,offy): (T,T,T,T)) -> (usize,usize)
    where
        T: Div<Output = T> + Sub<Output = T>,
        T: Into<u64>,
{
    (((x - offx) / csizex).into() as usize, ((y - offy) / csizey).into() as usize)
}

pub type ChunkPoly<T> = (u64,u64,Vec<PolyTriangle<T>>);
pub type ChunksPoly<T> = Vec<ChunkPoly<T>>;

pub fn chunkify_polytriangles<T>(cuts: u8, gbb: BB<T>, polygons: Vec<PolyTriangle<T>>) -> ChunksPoly<T>
    where
        T: Clone + Copy + Default + PartialEq + PartialOrd + Eq + std::hash::Hash,
        T: BoundingType + MinMax,
        T: Div<Output = T> + Add<Output = T> + Mul<Output = T> + Sub<Output = T>,
        T: Into<u64>,
        T: std::fmt::Display + std::fmt::Debug,
        u64: Ass<T>,
        u8: Ass<T>,
{
    let cuts_usize = cuts as usize;
    let cuts = cuts.ass();
    let mut grid: Vvec<PolyTriangle<T>> = vec![vec![]; cuts_usize * cuts_usize];
    let gwid = (gbb.1).0 - (gbb.0).0;
    let ghei = (gbb.1).1 - (gbb.0).1;
    let csizex = (gwid / cuts) + (1u64).ass();
    let csizey = (ghei / cuts) + (1u64).ass();
    let cinf = (csizex,csizey,(gbb.0).0,(gbb.0).1);
    println!("{:?} {} {}", gbb, csizex, csizey);
    for polygon in polygons{
        let mut localgrid = HashMap::new();
        for i in 0..polygon.indices.len() / 3{
            let ia = polygon.indices[i * 3];
            let ib = polygon.indices[i * 3 + 1];
            let ic = polygon.indices[i * 3 + 2];
            let va = polygon.vertices[ia as usize];
            let vb = polygon.vertices[ib as usize];
            let vc = polygon.vertices[ic as usize];
            let mut cells = vec![
                xy_to_chunk(va, cinf),
                xy_to_chunk(vb, cinf),
                xy_to_chunk(vc, cinf),
            ];
            cells.dedup();
            for (cx,cy) in cells{
                let (mut vertices,mut indices,mut indexmap) = if let Some((v,id,im)) = localgrid.remove(&(cx,cy)){
                    (v,id,im)
                }else{
                    (Vec::new(),Vec::new(),HashMap::new())
                };
                let mut localize = |vx|{
                    if let Some(ind) = indexmap.get(&vx){
                        indices.push(*ind as u16);
                    }else{
                        vertices.push(vx);
                        let ind = vertices.len() - 1;
                        indexmap.insert(vx, ind);
                        indices.push(ind as u16);
                    }
                };
                localize(va);
                localize(vb);
                localize(vc);
                localgrid.insert((cx,cy), (vertices,indices,indexmap));
            }
        }
        for ((cx,cy),(vertices,indices,_)) in localgrid{
            let mut pt = PolyTriangle{
                vertices,
                indices,
                style: polygon.style,
                bb: T::start_box(),
            };
            pt.stretch_bb();
            grid[cy * cuts_usize + cx].push(pt);
        }
    }
    let cuts_u64 = cuts_usize as u64;
    let mut chunks = Vec::new();
    for (i,vec) in grid.into_iter().enumerate(){
        let i = i as u64;
        let x = i % cuts_u64;
        let y = i / cuts_u64;
        chunks.push((x,y,vec));
    }
    chunks
}
