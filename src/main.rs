#![feature(slice_split_once)]
#![feature(portable_simd)]

mod array;
mod memmapped;
mod parse;

use std::{
    cmp::{max, min},
    collections::{BTreeMap, btree_map::Entry},
    io::{BufWriter, Write, stdout},
    path::PathBuf,
};

use hashbrown::{HashMap, hash_map::RawEntryMut};

use clap::Parser;

use crate::memmapped::{MemoryMappedFile, memchr, split_by};
use crate::parse::parse_temp;
use crate::{array::Array, memmapped::Lines};

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

    pub fn merge(&mut self, other: Self) {
        self.count += other.count;
        self.max = max(self.max, other.max);
        self.min = min(self.min, other.min);
        self.total += other.total;
    }
}

type Map = HashMap<Array, Record>;

fn process(values: &[u8]) -> Map {
    let mut map = Map::with_capacity(1_000);
    for line in Lines::new(values) {
        if line.is_empty() {
            continue;
        }

        let (station, temperature) = split_by(line, b';');

        let temperature = parse_temp(temperature);

        match map.raw_entry_mut().from_key(station) {
            RawEntryMut::Occupied(mut raw_occupied_entry_mut) => {
                raw_occupied_entry_mut.get_mut().update(temperature);
            }
            RawEntryMut::Vacant(raw_vacant_entry_mut) => {
                raw_vacant_entry_mut.insert(station.into(), Record::new(temperature));
            }
        }
    }

    map
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let file = MemoryMappedFile::open(args.path)?;
    let nthreads = std::thread::available_parallelism()?;
    let mut map: BTreeMap<Array, Record> = BTreeMap::new();

    std::thread::scope(|scope| {
        let values = file.as_ref();
        let (tx, rx) = std::sync::mpsc::sync_channel(nthreads.get());
        let chunk_size = values.len() / nthreads;
        let mut current = 0;
        for _ in 0..nthreads.get() {
            let start = current;
            let end = (current + chunk_size).min(values.len());
            let end = if end == values.len() {
                values.len()
            } else {
                match memchr(&values[end..], b'\n') {
                    Some(new_line) => end + new_line + 1,
                    None => values.len(),
                }
            };

            let values = &values[start..end];
            current = end;
            let tx = tx.clone();
            scope.spawn(move || tx.send(process(values)));
        }

        drop(tx);

        for records in rx {
            for (k, v) in records {
                match map.entry(k) {
                    Entry::Vacant(none) => {
                        none.insert(v);
                    }
                    Entry::Occupied(some) => {
                        let stat = some.into_mut();
                        stat.merge(v);
                    }
                }
            }
        }
    });

    let mut stats = map.iter().peekable();

    let mut writer = BufWriter::new(stdout().lock());

    write!(writer, "{{")?;

    while let Some((station, record)) = stats.next() {
        write!(
            writer,
            "{}={}/{}/{}",
            unsafe { std::str::from_utf8_unchecked(station) },
            (record.min as f64) / 10.0,
            ((record.total as f64) / 10.0) / record.count as f64,
            (record.max as f64) / 10.0
        )?;

        if stats.peek().is_some() {
            write!(writer, ", ")?;
        }
    }

    write!(writer, "}}")?;

    Ok(())
}
