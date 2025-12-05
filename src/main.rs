use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    path: PathBuf,
}

struct Record {
    count: usize,
    total: f64,
    min: f64,
    max: f64,
}

impl Record {
    pub fn new(count: usize, total: f64, min: f64, max: f64) -> Self {
        Self {
            count,
            total,
            min,
            max,
        }
    }

    pub fn update(&mut self, value: f64) {
        self.count += 1;
        self.max = self.max.max(value);
        self.min = self.min.min(value);
        self.total += value;
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let file = File::open(args.path)?;
    let reader = BufReader::new(file);

    let mut map = HashMap::<String, Record>::new();

    for line in reader.lines() {
        let line = line?;

        if line.trim().is_empty() {
            continue;
        }

        let (station, temperature) = line
            .split_once(';')
            .ok_or(anyhow::anyhow!("Invalid separator"))?;

        let temperature: f64 = temperature.trim().parse()?;

        if let Some(record) = map.get_mut(station) {
            record.update(temperature);
        } else {
            map.insert(
                station.to_string(),
                Record::new(1, temperature, temperature, temperature),
            );
        }
    }

    print!("{{");

    let mut stats = map.into_iter().collect::<Vec<_>>();
    stats.sort_by(|v1, v2| v1.0.cmp(&v2.0));

    let mut stats = stats.iter().peekable();

    while let Some((station, record)) = stats.next() {
        print!(
            "{station}={}/{}/{}",
            record.min,
            record.total / record.count as f64,
            record.max
        );

        if stats.peek().is_some() {
            print!(", ");
        }
    }

    print!("}}");

    Ok(())
}
