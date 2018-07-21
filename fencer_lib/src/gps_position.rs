//! Contains our type for GPS/WGS84 coordinates with the corresponding methods.

use distance::{fcc_approximation, ConvertError, Meter};
use exif;
use exif::Field;
use std;
use std::path::Path;

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

    // Todo: Replace this ugly unwraps.
    pub fn from_image_path(path: &AsRef<Path>) -> Result<GPSPosition, ConvertError> {
        let path = path.as_ref();
        let file = std::fs::File::open(path).unwrap();
        let reader = exif::Reader::new(&mut std::io::BufReader::new(&file));
        if let Err(e) = &reader {
            match e {
                exif::Error::NotFound { .. } => {
                    return Err(ConvertError::ExifNotFound {
                        path: Some(path.to_string_lossy().to_string()),
                    })
                }
                _ => {
                    return Err(ConvertError::ExifNotFound {
                        path: Some(path.to_string_lossy().to_string()),
                    })
                }
            }
        }
        match GPSPosition::from_exif(reader.unwrap().fields()) {
            Ok(pos) => Ok(pos),
            Err(_) => Err(ConvertError::ExifNotFound {
                path: Some(path.to_string_lossy().to_string()),
            }),
        }
    }

    /// Try to parse `GPSPosition` from EXIF data of a image.
    // Todo: Try also negative values. Like photos form africa.
    pub fn from_exif(fields: &[Field]) -> Result<GPSPosition, ConvertError> {
        let mut lat = None;
        let mut lon = None;
        for f in fields
            .into_iter()
            .filter(|x| !x.thumbnail && x.tag.context() == exif::Context::Gps)
        {
            match f.tag {
                // Todo: convert `rational` into floats and save them so that we can create a struct.
                exif::Tag::GPSLatitude => {
                    lat = Some(value_to_float(&f.value)?);
                }
                exif::Tag::GPSLongitude => {
                    lon = Some(value_to_float(&f.value)?);
                }
                _ => { /* default = noop*/ }
            }
        }
        if lat.is_some() && lon.is_some() {
            Ok(GPSPosition {
                lat: lat.unwrap(),
                lon: lon.unwrap(),
            })
        } else {
            Err(ConvertError::ExifNotFound { path: None })
        }
    }

    pub fn parse_from_string(input: &str) -> Result<GPSPosition, ConvertError> {
        let mut inp = String::from(input);
        inp = inp.replace(" ", ""); // remove whitespace
        let splited: Vec<&str> = inp.split(|x| x == ',' || x == ';').collect();
        if splited.len() != 2 {
            return Err(ConvertError::UnrecognisedString {
                input: input.to_string(),
            });
        }

        let lat = splited[0].parse::<f64>();
        let lon = splited[1].parse::<f64>();

        match (lat, lon) {
            (Ok(lat), Ok(lon)) => Ok(GPSPosition { lat, lon }),
            _ => Err(ConvertError::UnrecognisedString {
                input: input.to_string(),
            }),
        }
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

    #[test]
    fn parse_string_1() {
        let inp = "46.605243, 14.296923";
        let pos = GPSPosition::parse_from_string(inp).unwrap();
        assert_eq!(KLAGENFURT_DD, pos);
    }

    #[test]
    fn parse_string_2() {
        let inp = "45.212291; -79.327838";
        let pos = GPSPosition::parse_from_string(inp).unwrap();
        assert_eq!(UTTERSON_DD, pos);
    }

    #[test]
    fn parse_string_neg_1() {
        let inp = "46.605243 14.296923";
        let pos = GPSPosition::parse_from_string(inp);
        assert_eq!(
            Err(ConvertError::UnrecognisedString {
                input: inp.to_string()
            }),
            pos
        );
    }

    #[test]
    fn parse_string_neg_2() {
        let inp = "45.212291;";
        let pos = GPSPosition::parse_from_string(inp);
        assert_eq!(
            Err(ConvertError::UnrecognisedString {
                input: inp.to_string()
            }),
            pos
        );
    }
}
