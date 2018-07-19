//! A libary specialy for my find pictures binary.
//!
//! Check if some picture is taken in a specific area.
//!
//! Maybe usefull things, like the `fcc_approximation`, will get later they own libary so that it
//! is more reusable.

// Todo: Sort out double entries here:
#![warn(
    missing_debug_implementations, missing_copy_implementations, trivial_casts,
    trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
    unused_qualifications, missing_copy_implementations, missing_debug_implementations,
    trivial_casts, trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
    unused_qualifications, unused_extern_crates, unused_results, missing_docs
)]

extern crate exif;

pub mod distance;
pub mod gps_position;

use gps_position::GPSPosition;

/// Testfunction to evaluate the exif lib.
pub fn read_exif() {
    let path = "test_data/img01.jpg";
    let file = std::fs::File::open(path).unwrap();
    let reader = exif::Reader::new(&mut std::io::BufReader::new(&file)).unwrap();
    for f in reader
        .fields()
        .into_iter()
        .filter(|x| !x.thumbnail && x.tag.context() == exif::Context::Gps)
    {
        println!(
            "{} {} {} - {:?}",
            f.thumbnail,
            f.tag,
            f.value.display_as(f.tag),
            f.value
        );
    }

    GPSPosition::from_exif(reader.fields());
}

#[cfg(test)]
mod tests {
    use super::*;

    const IMG01: &str = "../test_data/img01.jpg";

    // Todo: Write tests for the other images. Also search non PosPos images.
    #[test]
    fn img01_into_range() {
        let position_test = GPSPosition {
            lat: 46.617128472222,
            lon: 14.266543388888,
        };
        let position_image = GPSPosition::from_image_path(IMG01).unwrap();
        let dist = position_image.distance(&position_test).unwrap();
        println!("Distance: {}m", dist);
        assert_ne!(dist, 0.0);
        assert!(dist < 0.1);
    }

    #[test]
    fn img01_out_of_range() {
        let position_test = GPSPosition {
            lat: 47.061578,
            lon: 15.420153,
        };
        let position_image = GPSPosition::from_image_path(IMG01).unwrap();
        let dist = position_image.distance(&position_test).unwrap();
        println!("Distance: {}m", dist);
        assert!(dist < 133e3);
        assert!(dist > 100e3);
    }
}
