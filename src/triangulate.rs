use crate::data::PolygonZ;
use dlv_list::*;
use crate::data::*;
use std::cmp::Ordering;
use std::ops::{Add,Sub,Div,Mul};

pub struct PolyTriangle<T>{
    vertices: Vec<(T,T)>,
    indices: Vec<usize>,
}

#[derive(Clone,PartialEq)]
struct PolyPoint<T>{
    point: P3<T>,
    reflex: bool,
    ear: bool,
    index: usize
}

pub fn test(){
    let mut polyzs = Vec::new();

    polyzs.push(PolygonZ{
        inners: vec![
            vec![
                (9.0,2.0,0.0),
                (9.0,8.0,0.0),
                (5.0,8.0,0.0),
                (5.0,2.0,0.0),
            ],
            vec![
                (3.0,4.0,0.0),
                (2.5,5.0,0.0),
                (2.0,4.0,0.0),
            ],
            vec![
                (7.5,4.0,0.0),
                (7.0,5.0,0.0),
                (6.5,4.0,0.0),
            ]
        ],
        outers: vec![
            vec![
                (6.0,3.0,0.0),
                (6.0,7.0,0.0),
                (8.0,7.0,0.0),
                (8.0,3.0,0.0)
            ],
            vec![
                (0.0,0.0,0.0),
                (0.0,10.0,0.0),
                (10.0,10.0,0.0),
                (10.0,0.0,0.0)
            ]
        ],
        bb: ((0.0,0.0,0.0),(0.0,0.0,0.0))
    });
    let res = triangulate(polyzs);

    for polyt in res{
        print!("vertices: ");
        for vert in polyt.vertices{print!("({},{}),",vert.0,vert.1)}
        println!();
        print!("indices: ");
        let mut i = 0;
        for index in polyt.indices{
            if i == 0 {print!("),(")}
            print!("{},",index);
            i = (i+1)%3;
        }
        println!();
    }
}

pub fn triangulate<T>(polyzs: Vec<PolygonZ<T>>) -> Vec<PolyTriangle<T>>
where
    T: Mul<Output = T> + Div<Output = T> + Add<Output = T> + Sub<Output = T> + PartialOrd + Copy + Default + MinMax,
    f64: Into<T>
{
    let mut res = Vec::new();
    for polygon in polyzs{
        let grouped_polygons = group_polygons(polygon);

        for (mut outer,inners) in grouped_polygons{
            let mut vertices = merge_inner(&mut outer, inners);
            vertices.dedup();
            if vertices[0] == vertices[vertices.len()-1]{vertices.pop();}
            let cur_indices = make_indices(&vertices);

            let mut p2vertices = Vec::new();
            for (x,y,_) in vertices{
                p2vertices.push((x,y));
            }
            res.push(PolyTriangle{
                vertices: p2vertices,
                indices: cur_indices
            });
        }
    }
    res
}

fn group_polygons<T>(polygon: PolygonZ<T>) -> Vec<(Vec<P3<T>>, Vvec<P3<T>>)>
where
    T: Mul<Output = T> + Div<Output = T> + Add<Output = T> + Sub<Output = T> + PartialOrd + Copy,
    f64: Into<T>
{
    //polygon polygon.outers[i] is inside inside[i] other polygons
    let mut inside = Vec::new();

    let mut grouped_inners = Vec::new();

    for outer in &polygon.outers{
        let mut count = 0;
        for other_outer in &polygon.outers{
            if outer == other_outer {continue}
            if is_inside_polygon(other_outer.to_vec(), outer[0]){
                count+=1;
            }
        }
        inside.push(count);

        grouped_inners.push(Vec::new());
    }

    for inner in polygon.inners{
        //find outer polygon that's inside the most other outer polygons
        //this is the 'most inner outer polygon', in a way, so this
        //inner ring belongs to that
        let mut max = -1;
        let mut max_index = 0;
        for (i,outer) in polygon.outers.iter().enumerate(){
            if is_inside_polygon(outer.to_vec(), inner[0]) && inside[i] > max  {
                max = inside[i];
                max_index = i;
            }
        }

        if max == -1 {panic!("inner ring not inside any outer rings")}

        grouped_inners[max_index].push(inner);
    }
    polygon.outers.into_iter().zip(grouped_inners).collect()
}

fn merge_inner<T>(outer: &mut Vec<P3<T>>, mut inners: Vvec<P3<T>>) -> Vec<P3<T>>
where
    T: Mul<Output = T> + Div<Output = T> + Add<Output = T> + Sub<Output = T> + PartialOrd + Copy + Default + MinMax,
    f64: Into<T>
{
    //merge inner ring with highest x coordinate first (this one can defneitely to see the outer ring)
    inners.sort_by(|a, b| rightmost(a).partial_cmp(&rightmost(b)).unwrap_or(Ordering::Equal));
    inners.reverse();

     //merge rings one by one
    for inner in inners{
        //get rightmost point in inner ring
        let mut rightmost = (T::minv(),T::minv(),T::minv());
        let mut rightmost_index = 0;
        for (index,point) in inner.iter().enumerate(){
            if point.0 > rightmost.0 {
                rightmost = *point;
                rightmost_index = index;
            }
        }

        //calculate closest intersection with outer ring when going to the right
        let mut intersect = (T::default(),T::default(),T::default());
        let mut intersect_index = 0;
        let mut best_dis = T::maxv();

        let x3 = rightmost.0;
        let y3 = rightmost.1;
        for i in 0..outer.len(){
            let x1 = outer[i].0;
            let y1 = outer[i].1;
            let x2 = outer[(i + 1) % outer.len()].0;
            let y2 = outer[(i + 1) % outer.len()].1;

            let zero = 0.0.into();
            let one = 1.0.into();

            let t = (y3 - y1) / (y2 - y1);
            if t < zero || t > one {continue}

            let x = x1 + t * (x2 - x1);
            let cur_dis = x - x3;
            if cur_dis<zero || cur_dis >= best_dis {continue}

            best_dis = cur_dis;
            let z1 = outer[i].2;
            let z2 = outer[(i + 1) % outer.len()].2;
            let z = z1 + t * (z2 - z1);
            intersect_index = (i + 1) % outer.len();
            intersect = (x,y3,z);
        }

        let mut new_vertices= Vec::new();
        for (i,point) in outer.iter().enumerate(){
            if i == intersect_index{
                new_vertices.push(intersect);

                let mut k = rightmost_index;
                let mut step = 0;
                while step < inner.len(){
                    new_vertices.push(inner[k]);
                    k = (k+1)%inner.len();
                    step+=1;
                }

                new_vertices.push(inner[k]);
                new_vertices.push(intersect);
            }
            new_vertices.push(*point);
        }

        *outer = new_vertices.clone();
    }
    outer.to_vec()
}

fn rightmost<T>(inner: &[P3<T>]) -> T
where
    T: Mul<Output = T> + Div<Output = T> + Add<Output = T> + Sub<Output = T> + PartialOrd + Copy + MinMax,
    f64: Into<T>
{
    let mut rightmost = T::minv();
    for (x,_,_) in inner{
        if x > &rightmost{
            rightmost = *x;
        }
    }
    rightmost
}

fn make_indices<T>(vertices: &[P3<T>]) -> Vec<usize>
where
    T: Mul<Output = T> + Div<Output = T> + Add<Output = T> + Sub<Output = T> + PartialOrd + Copy,
    f64: Into<T>
{
    if vertices.len() < 3 {panic!("polygon can't have fewer than 3 sides")}

    let mut remaining_polygon = VecList::new();
    remaining_polygon.reserve(vertices.len());
    let mut orig_indices = Vec::new();
    for (i,point) in vertices.iter().enumerate(){
        let p = PolyPoint{
            point: *point,
            reflex: false,
            ear: false,
            index: i
        };
        orig_indices.push(remaining_polygon.push_back(p));
    }

    let mut i = 0;
    let clone = &remaining_polygon.clone();
    for value in remaining_polygon.iter_mut(){
        value.reflex = is_reflex(clone, orig_indices[i]);
        i+=1;
    }

    i=0;
    let clone2 = &remaining_polygon.clone();
    for value in remaining_polygon.iter_mut(){
        value.ear = is_ear(clone2, orig_indices[i]);
        i+=1;
    }

    let mut indices = Vec::new();

    let mut cur_index = orig_indices[0];

    let mut step = 0;
    while remaining_polygon.len() > 3 {
        step+=1;

        if step > vertices.len(){
            panic!("no ears left!");
        }

        let cur = remaining_polygon.get(cur_index).unwrap();
        if !cur.ear {
            cur_index = next_index_cyclic(&remaining_polygon, cur_index);
            continue
        }

        step = 0;

        indices.push(prev_cyclic(&remaining_polygon,cur_index).index);
        indices.push(cur.index);
        indices.push(next_cyclic(&remaining_polygon,cur_index).index);

        let prev_index = prev_index_cyclic(&remaining_polygon,cur_index);
        let next_index = next_index_cyclic(&remaining_polygon, cur_index);
        remaining_polygon.remove(cur_index);

        update(&mut remaining_polygon, prev_index);
        update(&mut remaining_polygon, next_index);

        cur_index = prev_index;
    }

    for point in remaining_polygon{
        indices.push(point.index);
    }
    indices
}

fn next_cyclic<T>(polygon: &VecList<T>, i: Index<T>) -> &T{
    if let Some(y) = polygon.get_next_index(i){
        polygon.get(y).unwrap()
    }else{
        polygon.front().unwrap()
    }
}

fn prev_cyclic<T>(polygon: &VecList<T>, i: Index<T>) -> &T{
    if let Some(y) = polygon.get_previous_index(i){
        polygon.get(y).unwrap()
    }else{
        polygon.back().unwrap()
    }
}

//cyclic is very slow and stupid, but don't know how to do better with this library
fn next_index_cyclic<T>(polygon: &VecList<T>, i: Index<T>) -> Index<T>{
    if let Some(y) = polygon.get_next_index(i){
        y
    }else{
        //return the first index
        let mut indices = polygon.indices();
        indices.next().unwrap()
    }
}

//cyclic is very slow and stupid, but don't know how to do better with this library
fn prev_index_cyclic<T>(polygon: &VecList<T>, i: Index<T>) -> Index<T>{
    if let Some(y) = polygon.get_previous_index(i){
        y
    }else{
        //return the last index
        let indices = polygon.indices();
        indices.last().unwrap()
    }
}

fn update<T>(polygon: &mut VecList<PolyPoint<T>>, i: Index<PolyPoint<T>>)
where
    T: Mul<Output = T> + Div<Output = T> + Add<Output = T> + Sub<Output = T> + PartialOrd + Copy,
    f64: Into<T>
{
    let mut ear = false;
    let mut reflex = false;
    let p = polygon.get(i).unwrap();
    if !p.reflex {
        ear = is_ear(polygon, i);
        //convex points will stay convex
    }
    else{
        //reflex points might become convex
        reflex = is_reflex(polygon, i);
        //..and might become an ear
        if !reflex{
            ear = is_ear(polygon, i);
        }
    }

    let mut p_mut = polygon.get_mut(i).unwrap();
    p_mut.reflex = reflex;
    p_mut.ear = ear;
}

fn is_reflex<T>(polygon: &VecList<PolyPoint<T>>, i: Index<PolyPoint<T>>) -> bool
where
    T: Mul<Output = T> + Sub<Output = T> + PartialOrd + Copy,
    f64: Into<T>
{
    let ax = prev_cyclic(polygon,i).point.0;
    let ay = prev_cyclic(polygon,i).point.1;
    let bx = polygon.get(i).unwrap().point.0;
    let by = polygon.get(i).unwrap().point.1;
    let cx = next_cyclic(polygon,i).point.0;
    let cy = next_cyclic(polygon,i).point.1;

    (bx - ax) * (cy - by) - (cx - bx) * (by - ay) > 0.0.into()
}

fn is_ear<T>(polygon: &VecList<PolyPoint<T>>, i: Index<PolyPoint<T>>) -> bool
where
    T: Mul<Output = T> + Div<Output = T> + Add<Output = T> + Sub<Output = T> + PartialOrd + Copy,
    f64: Into<T>
{
    //an ear is a point of a triangle with no other points inside
    let p = polygon.get(i).unwrap();
    if is_reflex(polygon,i) {return false}
    let p_prev = prev_cyclic(polygon,i);
    let p_next = next_cyclic(polygon,i);
    for node in polygon{
        if node == p_prev || node == p || node == p_next {continue}
        if is_inside_triangle(node.point, p_prev.point, p.point, p_next.point) {
            return false
        }
    }
    true
}

fn is_inside_triangle<T>(p:P3<T>,p0:P3<T>,p1:P3<T>,p2:P3<T>) -> bool
where
    T: Mul<Output = T> + Div<Output = T> + Add<Output = T> + Sub<Output = T> + PartialOrd + Copy,
    f64: Into<T>
{
    let x0 = p.0;
    let y0 = p.1;
    let x1 = p0.0;
    let y1 = p0.1;
    let x2 = p1.0;
    let y2 = p1.1;
    let x3 = p2.0;
    let y3 = p2.1;

    if  p==p0 || p==p1 || p==p2{
        return false
    }

    let zero = 0.0.into();
    let one = 1.0.into();

    let denominator = (y2 - y3)*(x1 - x3) + (x3 - x2)*(y1 - y3);
    let aa = ((y2 - y3)*(x0 - x3) + (x3 - x2)*(y0 - y3)) / denominator;
    let bb = ((y3 - y1)*(x0 - x3) + (x1 - x3)*(y0 - y3)) / denominator;
    let cc = one - aa - bb;


    //epsilon is needed because of how inner and outer polygons are merged because
    //there will be two exactly equal lines in the polygon, only in reversed order
    aa >= zero && aa <= one && bb >= zero && bb <= one && cc >= zero && cc <= one
}


fn is_inside_polygon<T>(polygon: Vec<P3<T>>, p: P3<T>)-> bool
where
    T: Mul<Output = T> + Div<Output = T> + Add<Output = T> + Sub<Output = T> + PartialOrd + Copy,
    f64: Into<T>
{
    //shoot a ray to the right and count how many times it intersects the polygon
    //even means outside, odd means inside
    let mut intersects = 0;
    for i in 0..polygon.len(){
        let x1 = polygon[i].0;
        let y1 = polygon[i].1;
        let x2 = polygon[(i + 1) % polygon.len()].0;
        let y2 = polygon[(i + 1) % polygon.len()].1;

        let zero = 0.0.into();
        let one = 1.0.into();

        let t = (p.1 - y1) / (y2 - y1);
        if t < zero || t > one {continue}
        let x = x1 + t * (x2 - x1);
        if x<p.0 {continue}

        //there was an intersection
        intersects+=1;
    }
    intersects % 2 == 1
}
