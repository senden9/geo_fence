#[macro_use]
extern crate criterion;
extern crate failure;
extern crate rayon;
extern crate walkdir;

extern crate fencer_lib;

use std::path::Path;

use rayon::prelude::*;
use walkdir::{DirEntry, WalkDir};

use fencer_lib::*;

use criterion::{Bencher, Criterion};

type Result<T> = ::std::result::Result<T, ::failure::Error>;

fn run_sequential<P>(path: P, max_distance: f64, ref_pos: &str) -> Result<()>
where
    P: AsRef<Path>,
{
    let ref_pos = GPSPosition::parse_from_string(ref_pos)?;
    for entry in WalkDir::new(path)
        .into_iter()
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap())
    {
        if is_jpg(&entry) {
            let dist_ok = check_image(&ref_pos, max_distance, entry.path())?;
            if dist_ok {
                //println!("{:?} - {}", entry, dist_ok);
            }
        }
    }
    Ok(())
}

fn run_parallel<P>(path: P, max_distance: f64, ref_pos: &str, pool_size: usize) -> Result<()>
where
    P: AsRef<Path>,
{
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(pool_size)
        .build()
        .unwrap();
    let ref_pos = GPSPosition::parse_from_string(ref_pos)?;
    let mut files = Vec::new();
    for entry in WalkDir::new(path)
        .into_iter()
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap())
    {
        if is_jpg(&entry) {
            files.push(entry);
        }
    }

    if files.is_empty() {
        panic!("No images found. No testing is possible");
    }

    let mut positive: Vec<_> = pool.scope(|_s| {
        files
            .par_iter()
            .filter_map(|entry| {
                let img_in_range = check_image(&ref_pos, max_distance, entry.path()).unwrap();
                if img_in_range {
                    Some(entry.path())
                } else {
                    None
                }
            })
            .collect()
    });

    positive.sort_unstable();
    /*
    positive
        .iter()
        .for_each(|x| println!("{}", x.to_string_lossy()));
    */

    Ok(())
}

fn is_jpg(entry: &DirEntry) -> bool {
    let is_file = entry.file_type().is_file();
    let right_ending = entry
        .file_name()
        .to_string_lossy()
        .to_lowercase()
        .ends_with(".jpg");
    is_file && right_ending
}

fn check_image(point: &GPSPosition, max_distance: f64, image: &Path) -> Result<bool> {
    let pos = GPSPosition::from_image_path(&image);
    if let Err(e) = &pos {
        match e {
            distance::ConvertError::ExifNotFound { path } => {
                //println!("Some problem with: {:?}", path);
                return Ok(false);
            }
            _ => {}
        }
    }
    let pos = pos?;
    let img_distance = pos.distance(point);
    if let Err(e) = &img_distance {
        match e {
            distance::ConvertError::ResultToLarge => return Ok(false),
            _ => {}
        }
    }
    let img_distance = img_distance?;

    Ok(img_distance < max_distance)
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function_over_inputs(
        "test images parallel",
        |b: &mut Bencher, pool_size: &usize| {
            b.iter(|| run_parallel(Path::new("../test_data"), 80.0, "12.5; -9.22", *pool_size))
        },
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 14, 16, 20, 24],
    );
    c.bench_function("test images sequential", |b| {
        b.iter(|| run_sequential(Path::new("../test_data"), 80.0, "12.5; -9.22"))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
