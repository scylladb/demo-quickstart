use crate::util::coords;
use geohash::encode;
pub fn get_geo_hash(idx: usize) -> ((f64, f64), String) {
    let lat_long = coords::LATLONGS[idx];
    let coord = geohash::Coord { x: lat_long.0, y: lat_long.1 };
    let geo_hash = encode(coord, 5).expect("Failed to encode");
    (lat_long, geo_hash)
}
