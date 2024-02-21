/* Krüpl Zsolt, 2023. jun */

use clap::{Arg, ArgAction, Command};
use memmap::{MmapMut, MmapOptions};
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
                .default_value("1024")
                .value_parser(clap::value_parser!(u64))
                .action(ArgAction::Set)
                .help("Testfile size in MB")
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

fn speedtest(fvec: &mut MmapMut, number: u32, readwrite: bool, async_opt: bool) {
    // write 4k in random place
    let mut rng = rand::thread_rng();
    let mut rndvec: Vec<usize> = vec![];
    for _ in 0..number {
        rndvec.push(rng.gen_range(0..fvec.len() / 4096));
    }

    let start = time::Instant::now();
    if readwrite {
        // read and write
        for rndnum in rndvec {
            let fill = ((rndnum as u32) << 4) + 1;
            for i in 0..1024 {
                let stval = ((fvec[rndnum * 4096 + 4 * i] as u32) << 24)
                    + ((fvec[rndnum * 4096 + 4 * i + 1] as u32) << 16)
                    + ((fvec[rndnum * 4096 + 4 * i + 2] as u32) << 8)
                    + (fvec[rndnum * 4096 + 4 * i + 3] as u32)
                    + fill;
                fvec[rndnum * 4096 + 4 * i] = (stval >> 24) as u8;
                fvec[rndnum * 4096 + 4 * i + 1] = (stval >> 16) as u8;
                fvec[rndnum * 4096 + 4 * i + 2] = (stval >> 8) as u8;
                fvec[rndnum * 4096 + 4 * i + 3] = stval as u8;
            }
            if !async_opt {
                fvec.flush().unwrap();
            }
        }
    } else {
        // write only
        for rndnum in rndvec {
            let fill = ((rndnum as u32) << 4) + 1;
            for i in 0..1024 {
                fvec[rndnum * 4096 + 4 * i] = (fill >> 24) as u8;
                fvec[rndnum * 4096 + 4 * i + 1] = (fill >> 16) as u8;
                fvec[rndnum * 4096 + 4 * i + 2] = (fill >> 8) as u8;
                fvec[rndnum * 4096 + 4 * i + 3] = fill as u8;
            }
            if !async_opt {
                fvec.flush().unwrap();
            }
        }
    }
    fvec.flush().unwrap();
    let difftime = time::Instant::now() - start;
    let msec_4k = difftime.as_micros() as f64 / 1000. / number as f64;
    println!(
        "--> {msec_4k:.3} msec/4k block write (iops: {:.1})",
        1000. / msec_4k
    );
}

fn main() {
    let arg = argument_parser();
    let mut mbps = 0.0;
    let file = if let Ok(file) = File::options().read(true).write(true).open(FNAME) {
        if file.metadata().unwrap().len() == 1024 * 1024 * arg.mbyte {
            file
        } else {
            newfile(FNAME, arg.mbyte)
        }
    } else {
        unsafe { libc::sync() };
        let start = time::Instant::now();
        let file = newfile(FNAME, arg.mbyte);
        let difftime = time::Instant::now() - start;
        mbps = arg.mbyte as f64 / difftime.as_micros() as f64 * 1_000_000.;
        file
    };
    // memmap
    let mut fvec = unsafe { MmapOptions::new().map_mut(&file).unwrap() };
    let print_len = fvec.len() as f64 / 1024. / 1024. / 1024.;
    println!(
        "\nFile length: {print_len:.2} GB, number of random position 4kbyte test: {}",
        arg.number
    );
    if mbps > 0.0 {
        println!("--> Linear write: {:.2} Mbyte/s  ({FNAME})", mbps);
    } else {
        println!("--> Linear write: file already exists ({FNAME})");
    }
    // Run test
    speedtest(&mut fvec, arg.number, arg.readwrite, arg.async_opt);
}
