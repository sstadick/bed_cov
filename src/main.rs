#[macro_use]
extern crate clap;
extern crate bedlib;
extern crate grep_cli;
use bio::data_structures::interval_tree::ArrayBackedIntervalTree;
use coitrees::{COITree, IntervalNode};
use rust_lapper::{Interval, Lapper};
use std::collections::HashMap;
use std::error::Error;
use std::io::prelude::*;

arg_enum! {
    #[derive(Debug)]
    enum Library {
        LapperLib,
        CoitreesLib,
        IITreeLib,
    }
}

#[derive(Debug)]
struct LapperLib {}

#[derive(Debug)]
struct CoitreesLib {}

#[derive(Debug)]
struct IITreeLib {}

/// Implement bec_cov for an library
trait Runnable {
    fn run(&self, file_a: &str, file_b: &str) -> Result<(), Box<dyn Error>>;
}

// TODO: There must be a better way
impl Runnable for Library {
    /// Select the run function based on Library type
    fn run(&self, file_a: &str, file_b: &str) -> Result<(), Box<dyn Error>> {
        match *self {
            Library::LapperLib => LapperLib::run(file_a, file_b),
            Library::CoitreesLib => CoitreesLib::run(file_a, file_b),
            Library::IITreeLib => IITreeLib::run(file_a, file_b),
        }
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
    lib.run(file_a, file_b)?;
    Ok(())
}

impl LapperLib {
    fn run(file_a: &str, file_b: &str) -> Result<(), Box<dyn Error>> {
        let mut bed = HashMap::new();
        let mut buffer = String::new();
        let mut reader = bedlib::bed_reader::BufReader::open(file_a)?;
        while let Some((chr, start, stop)) = reader.read_line(&mut buffer)? {
            if !bed.contains_key(chr) {
                bed.insert(chr.to_string(), vec![]);
            }
            let start = start.parse::<u32>()?;
            let stop = stop.parse::<u32>()?;
            bed.get_mut(chr).unwrap().push(Interval {
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
        // Iter over B and get the values as we go
        let mut handle = grep_cli::stdout(termcolor::ColorChoice::Never);
        let mut reader = bedlib::bed_reader::BufReader::open(file_b)?;
        while let Some((chr, start, stop)) = reader.read_line(&mut buffer)? {
            if let Some(lapper) = lappers.get(chr) {
                let st0 = start.parse::<u32>()?;
                let en0 = stop.parse::<u32>()?;
                let mut cov_st = 0;
                let mut cov_en = 0;
                let mut cov = 0;
                let mut n = 0;
                for iv in lapper.find(st0, en0) {
                    n += 1;
                    let st1 = if iv.start > st0 { iv.start } else { st0 };
                    let en1 = if iv.stop < en0 { iv.stop } else { en0 };
                    if st1 > cov_en {
                        cov += cov_en - cov_st;
                        cov_st = st1;
                        cov_en = en1;
                    } else {
                        cov_en = if cov_en < en1 { en1 } else { cov_en };
                    }
                }
                cov += cov_en - cov_st;
                writeln!(handle, "{}\t{}\t{}\t{}\t{}", chr, st0, en0, n, cov)?;
            } else {
                // print the default stuff
                writeln!(handle, "{}\t{}\t{}\t0\t0", chr, start, stop)?;
            }
            buffer.clear();
        }
        Ok(())
    }
}

/// COItree...is fast
impl CoitreesLib {
    fn run(file_a: &str, file_b: &str) -> Result<(), Box<dyn Error>> {
        let mut bed = HashMap::new();
        let mut buffer = String::new();
        let mut reader = bedlib::bed_reader::BufReader::open(file_a)?;
        while let Some((chr, start, stop)) = reader.read_line(&mut buffer)? {
            if !bed.contains_key(chr) {
                bed.insert(chr.to_string(), vec![]);
            }
            let start = start.parse::<i32>()?;
            let stop = stop.parse::<i32>()?;
            bed.get_mut(chr)
                .unwrap()
                .push(IntervalNode::new(start, stop, ()));
        }
        // Convert to hash of lappers
        let mut coitrees = HashMap::new();
        for (key, value) in bed.into_iter() {
            coitrees.insert(key, COITree::new(value));
        }
        // Iter over B and get the values as we go
        let mut handle = grep_cli::stdout(termcolor::ColorChoice::Never);
        let mut reader = bedlib::bed_reader::BufReader::open(file_b)?;
        while let Some((chr, start, stop)) = reader.read_line(&mut buffer)? {
            if let Some(coitree) = coitrees.get(chr) {
                let st0 = start.parse::<i32>()?;
                let en0 = stop.parse::<i32>()?;
                let mut cov_st = 0;
                let mut cov_en = 0;
                let mut cov = 0;
                let mut n = 0;
                coitree.query(st0, en0, |iv| {
                    n += 1;
                    let st1 = if iv.first > st0 { iv.first } else { st0 };
                    let en1 = if iv.last < en0 { iv.last } else { en0 };
                    if st1 > cov_en {
                        cov += cov_en - cov_st;
                        cov_st = st1;
                        cov_en = en1;
                    } else {
                        cov_en = if cov_en < en1 { en1 } else { cov_en };
                    }
                });
                cov += cov_en - cov_st;
                writeln!(handle, "{}\t{}\t{}\t{}\t{}", chr, st0, en0, n, cov)?;
            } else {
                // print the default stuff
                writeln!(handle, "{}\t{}\t{}\t0\t0", chr, start, stop)?;
            }
            buffer.clear();
        }
        Ok(())
    }
}

/// IITree is rust-bios implementation of cgranges
impl IITreeLib {
    fn run(file_a: &str, file_b: &str) -> Result<(), Box<dyn Error>> {
        let mut bed = HashMap::new();
        let mut buffer = String::new();
        let mut reader = bedlib::bed_reader::BufReader::open(file_a)?;
        while let Some((chr, start, stop)) = reader.read_line(&mut buffer)? {
            if !bed.contains_key(chr) {
                bed.insert(chr.to_string(), ArrayBackedIntervalTree::new());
            }
            let start = start.parse::<u32>()?;
            let stop = stop.parse::<u32>()?;
            bed.get_mut(chr).unwrap().insert(start..stop, ());
        }
        // Convert to hash of lappers
        for (_, value) in bed.iter_mut() {
            value.index();
        }
        // Iter over B and get the values as we go
        let mut handle = grep_cli::stdout(termcolor::ColorChoice::Never);
        let mut reader = bedlib::bed_reader::BufReader::open(file_b)?;
        while let Some((chr, start, stop)) = reader.read_line(&mut buffer)? {
            if let Some(tree) = bed.get(chr) {
                let st0 = start.parse::<u32>()?;
                let en0 = stop.parse::<u32>()?;
                let mut cov_st = 0;
                let mut cov_en = 0;
                let mut cov = 0;
                let mut n = 0;
                for entry in tree.find(st0..en0) {
                    let iv = entry.interval();
                    n += 1;
                    let st1 = if iv.start > st0 { iv.start } else { st0 };
                    let en1 = if iv.end < en0 { iv.end } else { en0 };
                    if st1 > cov_en {
                        cov += cov_en - cov_st;
                        cov_st = st1;
                        cov_en = en1;
                    } else {
                        cov_en = if cov_en < en1 { en1 } else { cov_en };
                    }
                }
                cov += cov_en - cov_st;
                writeln!(handle, "{}\t{}\t{}\t{}\t{}", chr, st0, en0, n, cov)?;
            } else {
                // print the default stuff
                writeln!(handle, "{}\t{}\t{}\t0\t0", chr, start, stop)?;
            }
            buffer.clear();
        }
        Ok(())
    }
}
