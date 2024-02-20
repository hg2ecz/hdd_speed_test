/* Krüpl Zsolt, 2023. jun */

use clap::{Arg, ArgAction, Command};
use memmap::MmapOptions;
use rand::prelude::*;
use std::fs::File;
use std::io::Write;
use std::time;

const FNAME: &str = "testfile.dat";

struct Arguments {
    mbyte: u64,
    async_opt: bool,
    readwrite: bool,
    number: u32,
}

fn argument_parser() -> Arguments {
    let matches = Command::new("Filesystem 4 kbyte RW test.")
        .author("Zsolt Krüpl, hg2ecz@ham.hu")
        .version("Version: 0.2.0")
        .arg(
            Arg::new("size")
                .short('s')
                .long("size")
                .default_value("1000")
                .value_parser(clap::value_parser!(u64))
                .action(ArgAction::Set)
                .help("Testfile size in MB (default 1024)")
                .required(true),
        )
        .arg(
            Arg::new("async")
                .short('a')
                .long("async")
                .action(ArgAction::SetTrue)
                .help("Async write (default: sync)"),
        )
        .arg(
            Arg::new("readwrite")
                .short('r')
                .long("readwrite")
                .action(ArgAction::SetTrue)
                .help("Readwrite (default: write only)"),
        )
        .arg(
            Arg::new("number")
                .short('n')
                .long("number")
                .default_value("1000")
                .value_parser(clap::value_parser!(u32))
                .action(ArgAction::Set)
                .help("Number of test (default: 1000)"),
        )
        .get_matches();

    Arguments {
        mbyte: *matches.get_one("size").unwrap(),
        async_opt: *matches.get_one("async").unwrap(),
        readwrite: *matches.get_one("readwrite").unwrap(),
        number: *matches.get_one("number").unwrap(),
    }
}

fn newfile(fname: &str, size_mb: u64) -> File {
    let mut file = File::options()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(fname)
        .unwrap();
    let data = [0u8; 1024 * 1024];
    for _ in 0..size_mb {
        file.write_all(&data).unwrap();
    }
    file.flush().unwrap();
    unsafe { libc::sync() };
    file
}

fn main() {
    let arg = argument_parser();
    let mut rng = rand::thread_rng();
    let file = if let Ok(file) = File::options().read(true).write(true).open(FNAME) {
        if file.metadata().unwrap().len() == 1024 * 1024 * arg.mbyte {
            file
        } else {
            newfile(FNAME, arg.mbyte)
        }
    } else {
        newfile(FNAME, arg.mbyte)
    };
    // memmap
    let mut fvec = unsafe { MmapOptions::new().map_mut(&file).unwrap() };
    println!(
        "{:.2} GB hosszú fájl és {} alkalommal beleírva 4k",
        fvec.len() as f64 / 1024. / 1024. / 1024.,
        arg.number
    );

    // write 4k in random place
    let mut rndvec: Vec<usize> = vec![];
    for _ in 0..arg.number {
        rndvec.push(rng.gen_range(0..arg.mbyte as usize * 1024 / 4));
    }

    let start = time::Instant::now();
    if arg.readwrite {
        // read and write
        for rndnum in rndvec {
            for i in 0..4096 {
                fvec[rndnum * 4096 + i] += rndnum as u8;
            }
            if !arg.async_opt {
                fvec.flush().unwrap();
            }
        }
    } else {
        // write only
        for rndnum in rndvec {
            for i in 0..4096 {
                fvec[rndnum * 4096 + i] = rndnum as u8;
            }
            if !arg.async_opt {
                fvec.flush().unwrap();
            }
        }
    }
    fvec.flush().unwrap();
    let difftime = time::Instant::now() - start;

    println!(
        "{:.3} msec/4k block írás",
        difftime.as_micros() as f64 / 1000. / arg.number as f64
    );
}
