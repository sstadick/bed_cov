#[macro_use]
extern crate clap;
extern crate grep_cli;
use bio::data_structures::interval_tree::ArrayBackedIntervalTree;
use bio::utils;
use enum_dispatch::enum_dispatch;
use rust_lapper::Lapper;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;

// arg_enum! {
#[enum_dispatch(Runnable)]
#[derive(Debug)]
enum Library {
    LapperLib,
    CoitreesLib,
    IITreeLib,
}
// }

#[derive(Debug)]
struct LapperLib {}

#[derive(Debug)]
struct CoitreesLib {}

#[derive(Debug)]
struct IITreeLib {}

#[enum_dispatch]
trait Runnable {
    fn run(&self, file_a: &str, file_b: &str);
}

impl Runnable for LapperLib {
    fn run(&self, file_a: &str, file_b: &str) -> Result<(), Box<dyn Error>> {
        let mut bed = HashMap::new();
        let mut bederator = BedErator::new(file_a)?;
        while let Some((chr, start, stop)) = bederator.next() {
            if !bed.contains_key(chr) {
                bed.insert(chr.to_string(), vec![]);
            }
            let start = start.parse::<u32>().unwrap();
            let stop = stop.parse::<u32>().unwrap();
            bed.get_mut(chr).unwrap().push(rust_lapper::Interval {
                start,
                stop,
                val: (),
            });
        }
        // Convert to hash of lappers
        let mut lappers = HashMap::new();
        for (key, value) in bed.into_iter() {
            lappers.insert(key, Lapper::new(value));
        }
        println!("{:?}", lappers);
        Ok(())
    }
}

impl Runnable for CoitreesLib {
    fn run(&self, file_a: &str, file_b: &str) {
        panic!();
    }
}

impl Runnable for IITreeLib {
    fn run(&self, file_a: &str, file_b: &str) {
        panic!();
    }
}

struct BedErator<'a> {
    reader: BufReader<File>,
    buffer: String,
}

impl<'a> BedErator<'a> {
    fn new(file: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file)?;
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        Ok(BedErator { reader, buffer })
    }
}
struct BedLine<'a> {
    chr: &'a str
    start: &'a str,
    stop: &'a str,
}
impl<'a> Iterator for BedErator<'a> {
    type Item = BedLine<'a>;
    fn next(&mut self) -> Option<BedLine<'a>> {
        self.buffer.clear();
        while self.reader.read_line(&mut self.buffer).unwrap() > 0 {
            let mut iter = self.buffer[..self.buffer.len() - 1].split('\t');
            let chr = iter.next().unwrap();
            let start = iter.next().unwrap();
            let stop = iter.next().unwrap();
            return Some(BedLine{chr, start, stop});
        }
        None
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap_app!(bed_cov =>
        (version: crate_version!())
        (author: crate_authors!())
        (@arg library: -l --library +takes_value +required possible_values(&Library::variants()) "Which library to us")
        (@arg file_a: -a +takes_value +required "Path to input file a")
        (@arg file_b: -b +takes_value +required "Path to input file b")
    )
    .get_matches();
    let file_a = matches.value_of("file_a").unwrap();
    let file_b = matches.value_of("file_b").unwrap();
    let lib = value_t!(matches.value_of("library"), Library).unwrap();
    lib.run(file_a, file_b);

    // Read in all of file 1 into hash / lapper structure
    // let mut bed = HashMap::new();
    // let file = File::open(file_a)?;
    // let mut reader = BufReader::new(file);
    // let mut buffer = String::new();
    // while reader.read_line(&mut buffer).unwrap() > 0 {
    //     let mut iter = buffer[..buffer.len() - 1].split('\t');
    //     let chr = iter.next().unwrap();
    //     let start = iter.next().unwrap().parse::<u32>().unwrap();
    //     let stop = iter.next().unwrap().parse::<u32>().unwrap();

    //     match library {
    //         Lapper => {
    //             if !bed.contains_key(chr) {
    //                 bed.insert(chr.to_string(), vec![]);
    //             }
    //             bed.get_mut(chr).unwrap().push(Interval {
    //                 start,
    //                 stop,
    //                 val: (),
    //             });
    //         }
    //         Coitrees => {}
    //         IITree => {
    //             if !bed.contains_key(chr) {
    //                 bed.insert(chr.to_string(), ArrayBackedIntervalTree::new());
    //             }
    //         }
    //     }
    //     if !bed.contains_key(chr) {
    //         // bed.insert(chr.to_string(), ArrayBackedIntervalTree::new());
    //         bed.insert(chr.to_string(), vec![]);
    //     }
    //     bed.get_mut(chr).unwrap().push(Interval {
    //         start,
    //         stop,
    //         val: true,
    //     });
    //     // bed.get_mut(chr).unwrap().insert(start..stop, ());
    //     buffer.clear();
    // }

    // // Convert to hash of lappers
    // let mut lappers = HashMap::new();
    // for (key, value) in bed.into_iter() {
    //     lappers.insert(key, Lapper::new(value));
    //     // value.index();
    // }

    // // Iter over B and get the values as we go
    // // let stdout = io::stdout();
    // // let mut handle = stdout.lock();
    // let mut handle = grep_cli::stdout(termcolor::ColorChoice::Never);
    // let file = File::open(file_b)?;
    // let mut reader = BufReader::new(file);
    // buffer.clear();
    // while reader.read_line(&mut buffer).unwrap() > 0 {
    //     let mut iter = buffer[..buffer.len() - 1].split('\t');
    //     let chr = iter.next().unwrap();
    //     if let Some(lapper) = lappers.get(chr) {
    //         let st0 = iter.next().unwrap().parse::<u32>().unwrap();
    //         let en0 = iter.next().unwrap().parse::<u32>().unwrap();
    //         let mut cov_st = 0;
    //         let mut cov_en = 0;
    //         let mut cov = 0;
    //         let mut n = 0;
    //         for iv in lapper.find(st0, en0) {
    //             n += 1;
    //             let st1 = if iv.start > st0 { iv.start } else { st0 };
    //             let en1 = if iv.stop < en0 { iv.stop } else { en0 };
    //             if st1 > cov_en {
    //                 cov += cov_en - cov_st;
    //                 cov_st = st1;
    //                 cov_en = en1;
    //             } else {
    //                 cov_en = if cov_en < en1 { en1 } else { cov_en };
    //             }
    //         }
    //         cov += cov_en - cov_st;
    //         writeln!(handle, "{}\t{}\t{}\t{}\t{}", chr, st0, en0, n, cov)?;
    //     } else {
    //         let start = iter.next().unwrap();
    //         let stop = iter.next().unwrap();
    //         // print the default stuff
    //         writeln!(handle, "{}\t{}\t{}\t0\t0", chr, start, stop)?;
    //     }
    //     buffer.clear();
    // }
    Ok(())
}
