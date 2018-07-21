extern crate fencer_lib;
#[macro_use]
extern crate structopt;
extern crate clap_verbosity_flag;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate failure;
extern crate rayon;
extern crate walkdir;

use clap_verbosity_flag::Verbosity;
use fencer_lib::*;
use rayon::prelude::*;
use std::path::Path;
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

type Result<T> = ::std::result::Result<T, ::failure::Error>;

/// Scan a directory to near pictures
#[derive(Debug, StructOpt)]
struct Cli {
    /// Radius in meter
    #[structopt(long = "radius", short = "r", default_value = "80.0")]
    radius: f64,
    /// Point to check
    #[structopt(long = "point", short = "p")]
    point: String,
    /// The file to read
    dir: String,
    /// Parallel implementation? Default is secqiential.
    #[structopt(long = "palallel", short = "x")]
    parallel: bool,
    #[structopt(flatten)]
    verbosity: Verbosity,
}

fn main() {
    let args: Cli = Cli::from_args();
    args.verbosity.setup_env_logger("geo_fencer").unwrap();
    trace!("Info-Args: {:?}", &args);
    debug!("this is a debug {}", "message");

    rayon::ThreadPoolBuilder::new()
        .num_threads(24)
        .build_global()
        .unwrap();

    let root = Path::new(&args.dir);
    //assert!(env::set_current_dir(&root).is_ok());

    if args.parallel {
        if let Err(e) = run_parallel(&root, args.radius, &args.point) {
            println!("{}", e);
        }
    } else {
        if let Err(e) = run_sequential(&root, args.radius, &args.point) {
            println!("{}", e);
        }
    }
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

fn run_sequential<P>(path: P, max_distance: f64, ref_pos: &str) -> Result<()>
where
    P: AsRef<Path>,
{
    println!("Run sequential code");
    let ref_pos = GPSPosition::parse_from_string(ref_pos)?;
    for entry in WalkDir::new(path)
        .into_iter()
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap())
    {
        if is_jpg(&entry) {
            let dist_ok = check_image(&ref_pos, max_distance, entry.path())?;
            if dist_ok {
                println!("{:?} - {}", entry, dist_ok);
            }
        }
    }
    Ok(())
}

fn run_parallel<P>(path: P, max_distance: f64, ref_pos: &str) -> Result<()>
where
    P: AsRef<Path>,
{
    println!("Run parallel code");
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
        error!("No .jpg files found.");
    }

    let mut positive: Vec<_> = files
        .par_iter()
        .filter_map(|entry| {
            let img_in_range = check_image(&ref_pos, max_distance, entry.path()).unwrap();
            if img_in_range {
                Some(entry.path())
            } else {
                None
            }
        })
        .collect();

    positive.sort_unstable();
    positive
        .iter()
        .for_each(|x| println!("{}", x.to_string_lossy()));

    Ok(())
}

fn check_image(point: &GPSPosition, max_distance: f64, image: &Path) -> Result<bool> {
    let pos = GPSPosition::from_image_path(&image);
    if let Err(e) = &pos {
        match e {
            distance::ConvertError::ExifNotFound { path } => {
                println!("Some problem with: {:?}", path);
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
