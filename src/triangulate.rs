use crate::data::PolygonZ;

pub struct PolyTriangles{
    vertices: Vec<(f64,f64)>,
    indices: Vec<usize>,
}

pub fn triangulate<T>(polyzs: Vec<PolygonZ<T>>) -> Vec<PolyTriangles>
    where T: Copy,
{
    let mut res = Vec::new();
    res
}
