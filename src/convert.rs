// these functions just change from x.y() to y(x);
// because the last conversion functions were copied from our kotlin codebase.
// I did not want to rearrange the code since its complicated, just let it be.
fn floor(v: f64) -> f64 { v.floor() }
fn ln(v: f64) -> f64 { v.ln() }
fn cos(v: f64) -> f64 { v.cos() }
fn sin(v: f64) -> f64 { v.sin() }
fn atan(v: f64) -> f64 { v.atan() }
fn tan(v: f64) -> f64 { v.tan() }
fn sqrt(v: f64) -> f64 { v.sqrt() }
// I don't understand shit about how this works.
pub fn degree_to_utm((lon,lat): (f64,f64)) -> (usize,char,f64,f64){
    // some constants i pulled out of the code
    let (a,b,c,d,e,f) = (6.0,183.0,1.0,0.5,0.999_6,6_399_593.625);
    let (g,h,i,j,k,l) = (0.082_094_437_9_f64,2.0,3.0,100.0,0.01,0.006_739_496_742);
    let zone = floor(lon / a + 31.0);
    let letter = lat_to_utm_letter(lat);

    let deg = std::f64::consts::PI / 180.0;

    let mut easting = d * ln(
    (c + cos(lat * deg) * sin(lon * deg - (a * zone - b) * deg)) /
    (c - cos(lat * deg) * sin(lon * deg - (a * zone - b) * deg))
    ) * e * f / (c + g.powf(h) * cos(lat * deg).powf(h)).powf(d) * (c + g.powf(h
    ) / h * (d * ln((c + cos(lat * deg) * sin(lon * deg - (a * zone - b) * deg
    )) / (c - cos(lat * deg) * sin(lon * deg - (a * zone - b) * deg))
    )).powf(h) * cos(lat * deg).powf(h) / i) + 500_000.0;
    easting = (easting * j).round() * k;
    let mut northing = (atan(
    tan(lat * deg) / cos(lon * deg - (a * zone - b) * deg)
    ) - lat * deg) * e * f / sqrt(
    c + l * cos(lat * deg).powf(h)
    ) * (c + l / h * (d * ln((c + cos(lat * deg) * sin(lon * deg - (a * zone - b) * deg)) / (c - cos(
    lat * deg) * sin(lon * deg - (a * zone - b) * deg))
    )).powf(h) * cos(lat * deg).powf(h)) + e * f * (lat * deg - 0.005_054_622_556 * (lat
    * deg + sin(h * lat * deg
    ) / h) + 4.258_201_531e-05 * (i * (lat * deg + sin(h * lat * deg) / h) + sin(h * lat * deg
    ) * cos(lat * deg).powf(h)) / 4.0 - 1.674_057_895e-07 * (5.0 * (i * (lat * deg + sin(h * lat * deg) / h)     + sin(
    h * lat * deg) * cos(lat * deg).powf(h)) / 4.0 + sin(h * lat * deg) * cos(
    lat * deg).powf(h) * cos(lat * deg).powf(h)) / i);
    if letter < 'M' { northing += 10_000_000.0; }
    northing = (northing * j).round() * k;

    (zone as usize, letter, easting, northing)
}
// Just select the right zone letter from the latitude.
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
