//! This module implement a measure of distance between two points on the earth.
//! So a convert between two WGS84 points into the distance between this two points _along_
//! the earth surface.

use gps_position::GPSPosition;

/// Alisa for the default length data type. So that we can later easily switch the precision if necessary.
pub type Meter = f64;

/// Our error type for conversation operations.
#[derive(PartialEq, Debug, Clone, Fail)]
pub enum ConvertError {
    /// The result is to large to use safely. If our result is too large for
    /// the used approximation method then we have a low precision. Wa want avoid that.
    #[fail(display = "Result to large. Lost in precession with thad approximation method.")]
    ResultToLarge,
    /// Thrown when it was not possible to extract coordinates of a EXIF. Strongly possible
    /// that they are simple missing in a image.
    #[fail(display = "No EXIF data found in image. '{:?}'", path)]
    ExifNotFound { path: Option<String> },
    /// Thrown when there is unexpected content in a exif tag. Such as a ASCII typ in a GPS field.
    #[fail(display = "Found unexpected EXIF in a tag.")]
    UnexpectedExifContent,
    /// Can not parse a string into a `GPSPosition`.
    #[fail(display = "Can not parse this as a coordinate pair. '{}'", input)]
    UnrecognisedString { input: String },
}

/// Calculate the distance between two points on earth over the surface.
///
/// Should not be used for distances greater than 475 kilometers. A error will then emitted.
/// Mathematics taken from
/// [Code of Federal Regulations (annual edition). Title 47: Telecommunication. 73.208](https://www.gpo.gov/fdsys/pkg/CFR-2016-title47-vol4/pdf/CFR-2016-title47-vol4-sec73-208.pdf).
pub fn fcc_approximation(p1: &GPSPosition, p2: &GPSPosition) -> Result<Meter, ConvertError> {
    let delta_lat = p2.lat - p1.lat;
    let delta_lon = p2.lon - p1.lon;
    let lat_mean = (p1.lat + p2.lat) / 2.0; // in degree
    let lat_mean_rad = lat_mean.to_radians();
    let k1 =
        111.13209 - 0.56605 * (2.0 * lat_mean_rad).cos() + 0.00120 * (4.0 * lat_mean_rad).cos();
    let k2 = 111.41513 * lat_mean_rad.cos() - 0.09455 * (3.0 * lat_mean_rad).cos()
        + 0.00012 * (5.0 * lat_mean_rad).cos();

    let d = ((k1 * delta_lat).powi(2) + (k2 * delta_lon).powi(2)).sqrt(); // in kilometers

    if d > 475.0 {
        Err(ConvertError::ResultToLarge)
    } else {
        Ok(d * 1000.0) // return in meters
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KLAGENFURT: GPSPosition = GPSPosition {
        lat: 46.605243,
        lon: 14.296923,
    };
    const PRNJAVOR: GPSPosition = GPSPosition {
        lat: 44.861556,
        lon: 17.666873,
    };
    const UTTERSON: GPSPosition = GPSPosition {
        lat: 45.212291,
        lon: -79.327838,
    };
    const AAU_L7: GPSPosition = GPSPosition {
        lat: 46.614433,
        lon: 14.262985,
    };
    const AAU_L4: GPSPosition = GPSPosition {
        lat: 46.614739,
        lon: 14.263303,
    };

    #[test]
    fn fcc_ok() {
        //! Test just a OK. Not if the value is right.
        assert!(fcc_approximation(&KLAGENFURT, &PRNJAVOR).is_ok());
    }

    #[test]
    fn fcc_too_far() {
        //! Test if we are too far away with our points and therefore get an error.
        assert_eq!(
            fcc_approximation(&KLAGENFURT, &UTTERSON).unwrap_err(),
            ConvertError::ResultToLarge
        );
    }

    #[test]
    fn fcc_test_distance() {
        let calc_distance = fcc_approximation(&AAU_L4, &AAU_L7).unwrap();
        let known_distanc: f64 = 41.87;
        println!("Calc distance: {}", calc_distance);
        assert!((calc_distance - known_distanc).abs() < 0.1);
    }
}
