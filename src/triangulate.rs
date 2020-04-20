use crate::data::PolygonZ;
use dlv_list::*;
use crate::data::*;
use std::cmp::Ordering;
use std::borrow::Borrow;

pub struct PolyTriangles{
    vertices: Vec<(f64,f64)>,
    indices: Vec<usize>,
}

#[derive(Clone,PartialEq)]
struct PolyPoint{
    point: P3<f64>,
    reflex: bool,
    ear: bool,
    index: usize
}

pub fn triangulate(polyzs: Vec<PolygonZ<f64>>) -> Vec<PolyTriangles>
{
    for mut polygon in polyzs{
        //still need to group inner and outer rings
        let mut original_vertices = &mut polygon.outers[0];
        let mut vertices = merge_inner(&mut original_vertices, polygon.inners);
        //vertices.dedup();
        if vertices[0] == vertices[vertices.len()-1]{vertices.pop();}
        let indices = make_indices(&vertices);

        for (i,index) in indices.iter().enumerate(){
            println!("{}: {}", i, index);
        }
    }

    return Vec::new();
}

fn merge_inner(outer: &mut Vec<P3<f64>>, mut inners: Vvec<P3<f64>>) -> Vec<P3<f64>>{
    //merge inner ring with highest x coordinate first (this one can defneitely to see the outer ring)
    inners.sort_by(|a, b| rightmost(a).partial_cmp(rightmost(b)).unwrap_or(Ordering::Equal));
    inners.reverse();

     //merge rings one by one
    for inner in inners{
        //get rightmost point in inner ring
        let mut rightmost = (std::f64::MIN,std::f64::MIN,std::f64::MIN);
        let mut rightmost_index = 0;
        for (index,point) in inner.iter().enumerate(){
            if point.0 > rightmost.0 {
                rightmost = *point;
                rightmost_index = index;
            }
        }

        //calculate closest intersection with outer ring when going to the right
        let mut intersect = (0.0,0.0,0.0);
        let mut intersect_index = 0;
        let mut best_dis = std::f64::MAX;
        
        let x3 = rightmost.0;
        let y3 = rightmost.1;
        for i in 0..outer.len(){
            let x1 = outer[i].0;
            let y1 = outer[i].1;
            let x2 = outer[(i + 1) % outer.len()].0;
            let y2 = outer[(i + 1) % outer.len()].1;

            let t = (y3 - y1) / (y2 - y1);
            if t < 0.0 || t > 1.0 {continue}

            let x = x1 + t * (x2 - x1);
            let cur_dis = x - x3;
            if cur_dis<0.0 || cur_dis >= best_dis {continue}

            best_dis = cur_dis;
            let z1 = outer[i].2;
            let z2 = outer[(i + 1) % outer.len()].2;
            let z = z1 + t * (z2 - z1);
            intersect_index = (i + 1) % outer.len();
            intersect = (x,y3,z);
        }
        
        let mut new_vertices: Vec<P3<f64>> = Vec::new();
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
    return outer.to_vec();
}

fn rightmost(inner: &Vec<P3<f64>>) -> &f64{
    let mut rightmost = &std::f64::MIN;
    for (x,y,z) in inner{
        if x > rightmost{
            rightmost = x;
        }
    }
    rightmost
}

fn make_indices(vertices: &Vec<P3<f64>>) -> Vec<usize>{
    println!("len: {}", vertices.len());
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
    while(remaining_polygon.len() > 3){
        step+=1;

        if(step > vertices.len()){
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

        let mut prev_index = prev_index_cyclic(&remaining_polygon,cur_index);
        let mut next_index = next_index_cyclic(&remaining_polygon, cur_index);
        remaining_polygon.remove(cur_index);

        update(&mut remaining_polygon, prev_index);
        update(&mut remaining_polygon, next_index);

        cur_index = prev_index;
    }

    println!("triangulation steps: {}", step);

    for point in remaining_polygon{
        indices.push(point.index);
    }

    return indices;
}

fn next_cyclic<T>(polygon: &VecList<T>, i: Index<T>) -> &T{
    if let Some(y) = polygon.get_next_index(i){
        return polygon.get(y).unwrap();
    }else{
        return polygon.front().unwrap();
    }
}

fn prev_cyclic<T>(polygon: &VecList<T>, i: Index<T>) -> &T{
    if let Some(y) = polygon.get_previous_index(i){
        return polygon.get(y).unwrap();
    }else{
        return polygon.back().unwrap();
    }
}

//cyclic is very slow and stupid, but don't know how to do better with this library
fn next_index_cyclic<T>(polygon: &VecList<T>, i: Index<T>) -> Index<T>{
    if let Some(y) = polygon.get_next_index(i){
        return y;
    }else{
        //return the first index
        let indices = polygon.indices();
        let mut cur = i;//random value
        for index in indices{
            cur = index;
            break;
        }
        return cur;
    }
}

//cyclic is very slow and stupid, but don't know how to do better with this library
fn prev_index_cyclic<T>(polygon: &VecList<T>, i: Index<T>) -> Index<T>{
    if let Some(y) = polygon.get_previous_index(i){
        return y;
    }else{
        //return the last index
        let indices = polygon.indices();
        let mut cur = i; //random value
        for index in indices{
            cur = index;
        }
        return cur;
    }
}

fn update(polygon: &mut VecList<PolyPoint>, i: Index<PolyPoint>){
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

fn is_reflex(polygon: &VecList<PolyPoint>, i: Index<PolyPoint>) -> bool{
    let ax = prev_cyclic(polygon,i).point.0;
    let ay = prev_cyclic(polygon,i).point.1;
    let bx = polygon.get(i).unwrap().point.0;
    let by = polygon.get(i).unwrap().point.1;
    let cx = next_cyclic(polygon,i).point.0;
    let cy = next_cyclic(polygon,i).point.1;

    return (bx - ax) * (cy - by) - (cx - bx) * (by - ay) > 0.0;
}

fn is_ear(polygon: &VecList<PolyPoint>, i: Index<PolyPoint>) -> bool{
    //an ear is a point of a triangle with no other points inside
    let p = polygon.get(i).unwrap();
    if is_reflex(polygon,i) {return false}
    let p_prev = prev_cyclic(polygon,i);
    let p_next = next_cyclic(polygon,i);
    for node in polygon{
        if !node.reflex || node == p_prev || node == p || node == p_next {continue}
        if is_inside_triangle(node.point, p_prev.point, p.point, p_next.point) {
            return false
        }
    }
    return true
}

fn is_inside_triangle(p:P3<f64>,p0:P3<f64>,p1:P3<f64>,p2:P3<f64>) -> bool{
    let x = p.0;
    let y = p.1;
    let x1 = p0.0;
    let y1 = p0.1;
    let x2 = p1.0;
    let y2 = p1.1;
    let x3 = p2.0;
    let y3 = p2.1;

    let denominator = (y2 - y3)*(x1 - x3) + (x3 - x2)*(y1 - y3);
    let a = ((y2 - y3)*(x - x3) + (x3 - x2)*(y - y3)) / denominator;
    let b = ((y3 - y1)*(x - x3) + (x1 - x3)*(y - y3)) / denominator;
    let c = 1.0 - a - b;

    //epsilon is needed because of how inner and outer polygons are merged because
    //there will be two exactly equal lines in the polygon, only in reversed order
    let e = 0.0001;
    return a > 0.0+e && a < 1.0-e && b > 0.0+e && b < 1.0-e && c > 0.0+e && c < 1.0-e;
}