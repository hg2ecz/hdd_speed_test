/* Krüpl Zsolt, 2023. jun */

use clap::{Arg, ArgAction, Command};
use memmap::MmapOptions;
use rand::prelude::*;
use std::fs::File;
use std::io::Write;
use std::time;

const FNAME: &str = "tesztfile.dat";

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
    file
}

fn main() {
    let matches = Command::new("Filesystem RW test.")
        .version("Version: 0.1.0")
        .arg(
            Arg::new("size")
                .short('s')
                .long("size")
                .value_parser(clap::value_parser!(u64))
                .action(ArgAction::Set)
                .help("Size in MB (default 1024)"),
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
                .value_parser(clap::value_parser!(u32))
                .action(ArgAction::Set)
                .help("Number of test (default: 1000)"),
        )
        .get_matches();

    let mbyte: u64 = *matches.get_one("size").unwrap_or(&1024);
    let async_opt = *matches.get_one("async").unwrap_or(&false);
    let readwrite = *matches.get_one("readwrite").unwrap_or(&false);
    let number: u32 = *matches.get_one("number").unwrap_or(&1000);

    let mut rng = rand::thread_rng();
    let file = if let Ok(file) = File::options().read(true).write(true).open(FNAME) {
        if file.metadata().unwrap().len() == 1024 * 1024 * mbyte {
            file
        } else {
            newfile(FNAME, mbyte)
        }
    } else {
        newfile(FNAME, mbyte)
    };
    let mut fvec = unsafe { MmapOptions::new().map_mut(&file).unwrap() };
    println!(
        "{:.2} GB hosszú fájl és {number} alkalmommal beleírva 4k",
        fvec.len() as f64 / 1024. / 1024. / 1024.
    );

    // rnd helyen 4 kByte-ot írunk j alkalommal és megmérjük az idejét.
    let start = time::Instant::now();
    for j in 0..number {
        let rndnum: usize = rng.gen_range(0..mbyte as usize * 1024 / 4);
        if readwrite {
            // hozzáadjuk
            for i in 0..4096 {
                fvec[rndnum * 4096 + i] += j as u8;
            }
        } else {
            // csak írjuk
            for i in 0..4096 {
                fvec[rndnum * 4096 + i] = j as u8;
            }
        }
        if !async_opt {
            // kiírjuk!
            fvec.flush().unwrap();
        }
    }
    let diff = time::Instant::now() - start;
    println!(
        "{:.3} msec/4k block írás",
        diff.as_micros() as f64 / 1000. / number as f64
    );
}
