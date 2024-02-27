use clap::{Arg, ArgAction, Command};
use memmap::{MmapMut, MmapOptions};
use rand::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use std::thread;
use std::time;

const DIRNAME: &str = "hdd_speed_test_tmpdir";
const FILENAME: &str = "testfile.dat";
const CHUNK_N_4K: u64 = 1;
const CHUNKSIZE: u64 = 4096 * CHUNK_N_4K;

struct Arguments {
    mbyte: u64,
    async_opt: bool,
    readwrite: bool,
    wrnum: u32,
    threadnums: u64,
    keepfiles: bool,
}

fn argument_parser() -> Arguments {
    let matches = Command::new("Filesystem 4 kbyte RW test.")
        .author("Zsolt Krüpl, hg2ecz@ham.hu")
        .version("Version: 0.3.0")
        .arg(
            Arg::new("size")
                .short('s')
                .long("size")
                .default_value("1024")
                .value_parser(clap::value_parser!(u64))
                .action(ArgAction::Set)
                .help("Sum of testfile size in MB")
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
            Arg::new("wrnum")
                .short('n')
                .long("number")
                .default_value("1000")
                .value_parser(clap::value_parser!(u32))
                .action(ArgAction::Set)
                .help("how many block will be write"),
        )
        .arg(
            Arg::new("threadnums")
                .short('t')
                .long("threadnums")
                .default_value("1")
                .value_parser(clap::value_parser!(u64))
                .action(ArgAction::Set)
                .help("wrnum of threads"),
        )
        .arg(
            Arg::new("keepfiles")
                .short('k')
                .long("keepfiles")
                .action(ArgAction::SetTrue)
                .help("keepfiles (default: no)"),
        )
        .get_matches();

    Arguments {
        mbyte: *matches.get_one("size").unwrap(),
        async_opt: *matches.get_one("async").unwrap(),
        readwrite: *matches.get_one("readwrite").unwrap(),
        wrnum: *matches.get_one("wrnum").unwrap(),
        threadnums: *matches.get_one("threadnums").unwrap(),
        keepfiles: *matches.get_one("keepfiles").unwrap(),
    }
}

fn newfile(fname: &str, filesize: u64, filldata: &[u8], fillchunk: &[u8]) -> File {
    let mut file = File::options()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(fname)
        .unwrap();

    let mut remain = filesize as usize;
    while remain >= filldata.len() {
        file.write_all(filldata).unwrap();
        remain -= filldata.len();
    }
    while remain >= fillchunk.len() {
        file.write_all(fillchunk).unwrap();
        remain -= fillchunk.len();
    }
    file.flush().unwrap();
    unsafe { libc::sync() };
    file
}

fn speedtest_testfunc(fvec: &mut MmapMut, rndvec: &[usize], readwrite: bool, async_opt: bool) {
    let chunkusize = CHUNKSIZE as usize;
    if readwrite {
        // read and write
        for &rndnum in rndvec {
            let fill = ((rndnum as u32) << 4) + 1;
            for i in 0..chunkusize / 4 {
                let stval = (((fvec[rndnum * chunkusize + 4 * i] as u32) << 24)
                    + ((fvec[rndnum * chunkusize + 4 * i + 1] as u32) << 16)
                    + ((fvec[rndnum * chunkusize + 4 * i + 2] as u32) << 8)
                    + (fvec[rndnum * chunkusize + 4 * i + 3] as u32))
                    .wrapping_add(fill);
                fvec[rndnum * chunkusize + 4 * i] = (stval >> 24) as u8;
                fvec[rndnum * chunkusize + 4 * i + 1] = (stval >> 16) as u8;
                fvec[rndnum * chunkusize + 4 * i + 2] = (stval >> 8) as u8;
                fvec[rndnum * chunkusize + 4 * i + 3] = stval as u8;
            }
            if !async_opt {
                fvec.flush().unwrap();
            }
        }
    } else {
        // write only
        for &rndnum in rndvec {
            let fill = ((rndnum as u32) << 4) + 1;
            for i in 0..chunkusize / 4 {
                fvec[rndnum * chunkusize + 4 * i] = (fill >> 24) as u8;
                fvec[rndnum * chunkusize + 4 * i + 1] = (fill >> 16) as u8;
                fvec[rndnum * chunkusize + 4 * i + 2] = (fill >> 8) as u8;
                fvec[rndnum * chunkusize + 4 * i + 3] = fill as u8;
            }
            if !async_opt {
                fvec.flush().unwrap();
            }
        }
    }
    fvec.flush().unwrap();
}

fn create_files(filesize: u64, threadnums: u64) -> Option<f64> {
    let mut filldata = [0u8; 128 * 1024];
    let mut fillchunk = [0u8; CHUNKSIZE as usize];
    let mut rng = rand::thread_rng();
    for d in &mut filldata {
        *d = rng.gen();
    }
    for d in &mut fillchunk {
        *d = rng.gen();
    }

    // create files for tests
    let mut children = vec![];
    for i in 0..threadnums {
        children.push(thread::spawn(move || -> f64 {
            let mut mbps = 0.0;
            let dirfilename = format!("{DIRNAME}/{FILENAME}-{i:06}");
            if let Ok(file) = File::options().read(true).write(true).open(&dirfilename) {
                if file.metadata().unwrap().len() != filesize {
                    newfile(&dirfilename, filesize, &filldata, &fillchunk);
                }
            } else {
                unsafe { libc::sync() };
                let start = time::Instant::now();
                newfile(&dirfilename, filesize, &filldata, &fillchunk);
                let difftime = time::Instant::now() - start;
                mbps = filesize as f64 / 1024. / 1024. / difftime.as_micros() as f64 * 1_000_000.;
            };
            mbps
        }));
    }
    // join
    let mut mbps = 0.0;
    let mut mbps_ok = true;
    for c in children {
        let mbps_x = c.join().unwrap();
        if mbps_x == 0.0 {
            mbps_ok = false;
        }
        mbps += mbps_x;
    }
    if mbps_ok {
        Some(mbps)
    } else {
        None
    }
}

fn random_write_test(filesize: u64, arg: &Arguments) -> (usize, f64) {
    let mut rng = rand::thread_rng();
    let mut rndvec_common: Vec<usize> = vec![];
    let chunk_number = filesize / CHUNKSIZE;
    for _ in 0..arg.wrnum / arg.threadnums as u32 {
        rndvec_common.push(rng.gen_range(0..chunk_number as usize));
    }

    let readwrite = arg.readwrite;
    let async_opt = arg.async_opt;
    let mut children = vec![];
    let start = time::Instant::now();
    for i in 0..arg.threadnums {
        let rndvec = rndvec_common.clone();
        let dirfilename = format!("{DIRNAME}/{FILENAME}-{i:06}");
        children.push(thread::spawn(move || -> usize {
            let file = if let Ok(file) = File::options().read(true).write(true).open(&dirfilename) {
                if file.metadata().unwrap().len() == filesize {
                    file
                } else {
                    eprintln!("Error: filesize mismatch");
                    std::process::exit(-1);
                }
            } else {
                eprintln!("Error: no such file");
                std::process::exit(-1);
            };
            // memmap
            let mut fvec = unsafe { MmapOptions::new().map_mut(&file).unwrap() };
            speedtest_testfunc(&mut fvec, &rndvec, readwrite, async_opt);
            fvec.len()
        }));
    }
    // join
    let sum_len: usize = children.into_iter().map(|c| c.join().unwrap()).sum();

    let difftime = time::Instant::now() - start;
    let msec_4k = difftime.as_micros() as f64 / 1000. / arg.wrnum as f64; // !!!
    (sum_len, msec_4k)
}

fn remove_tmp_files(threadnums: u64) {
    // println!("Remove tmp files -> {DIRNAME}");
    for i in 0..threadnums {
        let dirfilename = format!("{DIRNAME}/{FILENAME}-{i:06}");
        if fs::remove_file(&dirfilename).is_err() {
            eprintln!("Remove error: {dirfilename} not exists!");
        }
    }
    if std::fs::remove_dir(DIRNAME).is_err() {
        eprintln!("--- Directory {DIRNAME} is not empty. Please remove manually! ---");
    }
}

fn main() {
    let arg = argument_parser();
    std::fs::create_dir_all(DIRNAME).unwrap();

    ctrlc::set_handler(move || {
        remove_tmp_files(arg.threadnums);
        std::process::exit(-1);
    })
    .expect("Error setting Ctrl-C handler");

    let chunk_number = 1024 * 1024 * arg.mbyte / CHUNKSIZE;
    let filesize = CHUNKSIZE * (chunk_number / arg.threadnums);

    let mbps_opt = create_files(filesize, arg.threadnums);
    let (sum_len, msec_4k) = random_write_test(filesize, &arg);

    println!("\nTotal testfile lenghts: {:.2} GB, {} pcs in total of random position {}kbyte test in {} threads.",
        sum_len as f64 / 1024. / 1024. / 1024., (arg.wrnum / arg.threadnums as u32) * arg.threadnums as u32,
        CHUNKSIZE / 1024, arg.threadnums);

    if let Some(mbps) = mbps_opt {
        println!("--> Linear write: {:.2} Mbyte/s  ({DIRNAME}/*)", mbps);
    } else {
        println!("--> Linear write: file already exists ({DIRNAME}/*)");
    }
    println!(
        "--> {msec_4k:.3} msec/4k block write (iops: {:.1})",
        1000. / msec_4k
    );

    if !arg.keepfiles {
        remove_tmp_files(arg.threadnums);
    } else {
        println!("TMP files remains in {DIRNAME} directory. Please remove manually.");
    }
    println!();
}
