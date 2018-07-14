//! Contains our type for GPS/WGS84 coordinates with the corresponding methods.

use distance::{fcc_approximation, ConvertError, Meter};

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

impl GPSPosition {
    /// Calculate the distance in meters between two points.
    pub fn distance(&self, other: &GPSPosition) -> Result<Meter, ConvertError> {
        fcc_approximation(self, other)
    }
}
