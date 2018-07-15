//! Contains our type for GPS/WGS84 coordinates with the corresponding methods.

use distance::{fcc_approximation, ConvertError, Meter};
use exif;
use exif::Field;

/// WGS84 Coordinates.
///
/// No validation or normalisation is done at the moment!
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct GPSPosition {
    /// Latitude of the coordinate in degrees.
    pub lat: f64,
    /// Longitude of the coordinate in degrees.
    pub lon: f64,
}

// Todo: Implement `std::convert::TryFrom` for EXIF when stable.
impl GPSPosition {
    /// Calculate the distance in meters between two points.
    pub fn distance(&self, other: &GPSPosition) -> Result<Meter, ConvertError> {
        fcc_approximation(self, other)
    }

    /// Try to parse `GPSPosition` from EXIF data of a image.
    // Todo: Try also negative values. Like photos form africa.
    pub fn from_exif(fields: &[Field]) -> Result<GPSPosition, ConvertError> {
        for f in fields
            .into_iter()
            .filter(|x| !x.thumbnail && x.tag.context() == exif::Context::Gps)
        {
            match f.tag {
                // Todo: convert `rational` into floats and save them so that we can create a struct.
                exif::Tag::GPSLatitude => {
                    let lat = value_to_float(&f.value)?;
                    println!("Found Lat! {:?} = {}", f.value, lat);
                }
                exif::Tag::GPSLongitude => {
                    let lon = value_to_float(&f.value)?;
                    println!("Found Long! {:?} = {}", f.value, lon);
                }
                _ => { /* default = noop*/ }
            }
        }
        Err(ConvertError::ExifNotFound)
    }
}

/// Convert a Value from Degree, Minute, Second into decimal degree.
// Todo: Write tests for this.
fn value_to_float(val: &exif::Value) -> Result<f64, ConvertError> {
    match val {
        exif::Value::Rational(rat) => {
            if rat.len() != 3 {
                // Expect Degree, Minute, Second
                return Err(ConvertError::UnexpectedExifContent);
            }
            let decimal_degree =
                rat[0].to_f64() + rat[1].to_f64() / 60.0 + rat[2].to_f64() / (60.0 * 60.0);
            Ok(decimal_degree)
        }
        _ => Err(ConvertError::UnexpectedExifContent),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Positive on both axes.
    const KLAGENFURT_DD: GPSPosition = GPSPosition {
        lat: 46.605243,
        lon: 14.296923,
    };
    // Todo: Test PosNeg (Utterson)
    const UTTERSON_DD: GPSPosition = GPSPosition {
        lat: 45.212291,
        lon: -79.327838,
    };
    // Todo: Find and test NegPos & NegNeg
    const CONVERT_EPS: f64 = 1.0e-8; // precession for tests.

    #[test]
    fn test_value_to_float_ok_pos_pos() {
        // Klagenfurt: 46° 36' 18.8748" 14° 17' 48.9228" E
        // Converted using https://www.latlong.net/lat-long-dms.html
        let klagenfurt_dms_lat = exif::Value::Rational(vec![
            exif::Rational { num: 46, denom: 1 },
            exif::Rational { num: 36, denom: 1 },
            exif::Rational {
                num: 188748,
                denom: 10000,
            },
        ]);
        let klagenfurt_dms_lon = exif::Value::Rational(vec![
            exif::Rational { num: 14, denom: 1 },
            exif::Rational { num: 17, denom: 1 },
            exif::Rational {
                num: 489228,
                denom: 10000,
            },
        ]);
        assert!(
            (KLAGENFURT_DD.lat - value_to_float(&klagenfurt_dms_lat).unwrap()).abs() < CONVERT_EPS
        );
        assert!(
            (KLAGENFURT_DD.lon - value_to_float(&klagenfurt_dms_lon).unwrap()).abs() < CONVERT_EPS
        );
    }
}
