//! A libary specialy for my find pictures binary.
//!
//! Check if some picture is taken in a specific area.
//!
//! Maybe usefull things, like the `fcc_approximation`, will get later they own libary so that it
//! is more reusable.

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
}

#[cfg(test)]
mod tests {
    use exif;
    use std;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
