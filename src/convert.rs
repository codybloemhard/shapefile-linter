fn floor(v: f64) -> f64 { v.floor() }
fn ln(v: f64) -> f64 { v.ln() }
fn cos(v: f64) -> f64 { v.cos() }
fn sin(v: f64) -> f64 { v.sin() }
fn atan(v: f64) -> f64 { v.atan() }
fn tan(v: f64) -> f64 { v.tan() }
fn sqrt(v: f64) -> f64 { v.sqrt() }

pub fn degree_to_utm((lat,lon): (f64,f64)) -> (usize,char,f64,f64){
    let zone = floor(lon / 6.0 + 31.0);
    let letter = lat_to_utm_letter(lat);

    let deg = std::f64::consts::PI / 180.0;

    let mut easting = 0.5 * ln(
    (1.0 + cos(lat * deg) * sin(lon * deg - (6.0 * zone - 183.0) * deg)) /
    (1.0 - cos(lat * deg) * sin(lon * deg - (6.0 * zone - 183.0) * deg))
    ) * 0.9996 * 6399593.62 / (1.0 + 0.0820944379_f64.powf(2.0) * cos(lat * deg).powf(2.0)).powf(0.5) * (1.0 + 0.0820944379_f64.powf(2.0
    ) / 2.0 * (0.5 * ln((1.0 + cos(lat * deg) * sin(lon * deg - (6.0 * zone - 183.0) * deg
    )) / (1.0 - cos(lat * deg) * sin(lon * deg - (6.0 * zone - 183.0) * deg))
    )).powf(2.0) * cos(lat * deg).powf(2.0) / 3.0) + 500000.0;
    easting = (easting * 100.0).round() * 0.01;
    let mut northing = (atan(
    tan(lat * deg) / cos(lon * deg - (6.0 * zone - 183.0) * deg)
    ) - lat * deg) * 0.9996 * 6399593.625 / sqrt(
    1.0 + 0.006739496742 * cos(lat * deg).powf(2.0)
    ) * (1.0 + 0.006739496742 / 2.0 * (0.5 * ln((1.0 + cos(lat * deg) * sin(lon * deg - (6.0 * zone - 183.0) * deg)) / (1.0 - cos(
    lat * deg) * sin(lon * deg - (6.0 * zone - 183.0) * deg))
    )).powf(2.0) * cos(lat * deg).powf(2.0)) + 0.9996 * 6399593.625 * (lat * deg - 0.005054622556 * (lat
    * deg + sin(2.0 * lat * deg
    ) / 2.0) + 4.258201531e-05 * (3.0 * (lat * deg + sin(2.0 * lat * deg) / 2.0) + sin(2.0 * lat * deg
    ) * cos(lat * deg).powf(2.0)) / 4.0 - 1.674057895e-07 * (5.0 * (3.0 * (lat * deg + sin(2.0 * lat * deg) / 2.0)     + sin(
    2.0 * lat * deg) * cos(lat * deg).powf(2.0)) / 4.0 + sin(2.0 * lat * deg) * cos(
    lat * deg).powf(2.0) * cos(lat * deg).powf(2.0)) / 3.0);
    if letter < 'M' { northing += 10000000.0; }
    northing = (northing * 100.0).round() * 0.01;

    return (zone as usize, letter, easting, northing)
}

fn lat_to_utm_letter(lat: f64) -> char{
    let letters = vec!['C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M',
        'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W'];
    let mut counter = -72;
    for l in letters{
        if lat < counter as f64 {
            return l;
        }
        counter += 8
    }
    'X'
}
