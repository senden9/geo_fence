# geo_fence
[![Build Status](https://travis-ci.com/senden9/geo_fence.svg?branch=master)](https://travis-ci.com/senden9/geo_fence)

Rust binary to serach Pictures (based on EXIF) in a specific region. 

Todo: Add a better readme ;)

```
$ geo_fence -h
Scan a directory to near pictures

USAGE:
    geo_fence [FLAGS] [OPTIONS] <dir> --point <point>

FLAGS:
    -h, --help         Prints help information
    -x, --palallel     Parallel implementation? Default is secqiential.
    -V, --version      Prints version information
    -v, --verbosity    Pass many times for more log output
                       
                       By default, it'll only report errors. Passing `-v` one time also prints warnings, `-vv` enables
                       info logging, `-vvv` debug, and `-vvvv` trace.

OPTIONS:
    -p, --point <point>      Point to check
    -r, --radius <radius>    Radius in meter [default: 80.0]

ARGS:
    <dir>    The file to read

```
