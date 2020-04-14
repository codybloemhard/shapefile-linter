use crate::data::PolygonZ;

pub struct PolyTriangles{
    vertices: Vec<(f64,f64)>,
    indices: Vec<usize>,
}

pub fn triangulate(polyzs: Vec<PolygonZ<f64>>) -> Vec<PolyTriangles>
{
    let mut res = Vec::new();
    res
}
