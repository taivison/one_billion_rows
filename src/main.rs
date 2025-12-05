#![feature(slice_split_once)]
#![feature(portable_simd)]

mod array;
mod memmapped;
mod parse;

use std::{
    cmp::{max, min},
    collections::HashMap,
    path::PathBuf,
};

use clap::Parser;

use crate::array::Array;
use crate::memmapped::{MemoryMappedFile, split_by};
use crate::parse::parse_temp;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    path: PathBuf,
}

struct Record {
    count: usize,
    total: i64,
    min: i16,
    max: i16,
}

impl Record {
    pub fn new(value: i16) -> Self {
        Self {
            count: 1,
            total: value as i64,
            min: value,
            max: value,
        }
    }

    pub fn update(&mut self, value: i16) {
        self.count += 1;
        self.max = max(self.max, value);
        self.min = min(self.min, value);
        self.total += value as i64;
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let file = MemoryMappedFile::open(args.path)?;

    let mut map = HashMap::<Array, Record>::with_capacity(10_000);

    for line in file.lines() {
        if line.is_empty() {
            continue;
        }

        let (station, temperature) = split_by(line, b';');

        let temperature = parse_temp(temperature);

        if let Some(record) = map.get_mut(station) {
            record.update(temperature);
        } else {
            map.insert(Array::new(station), Record::new(temperature));
        }
    }

    print!("{{");

    let mut stats = map.into_iter().collect::<Vec<_>>();
    stats.sort_by(|v1, v2| v1.0.cmp(&v2.0));

    let mut stats = stats.iter().peekable();

    while let Some((station, record)) = stats.next() {
        print!(
            "{}={}/{}/{}",
            unsafe { str::from_utf8_unchecked(station) },
            (record.min as f64) / 10.0,
            ((record.total as f64) / 10.0) / record.count as f64,
            (record.max as f64) / 10.0
        );

        if stats.peek().is_some() {
            print!(", ");
        }
    }

    print!("}}");

    Ok(())
}
