use std::fs::File;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::{env, fmt};

use criterion::measurement::Measurement;
use criterion::{black_box, BatchSize, BenchmarkGroup, Throughput};

#[cfg(feature = "osm-pbf-reader")]
pub mod impl_osm_pbf_reader;
#[cfg(feature = "osm-pbf-reader-pb4")]
pub mod impl_osm_pbf_reader_pb4;
#[cfg(feature = "osmpbf")]
pub mod impl_osmpbf;
#[cfg(feature = "osmpbfreader")]
pub mod impl_osmpbfreader;

pub const LICHTENSTEIN_PBF: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/liechtenstein-latest.osm.pbf");
pub const IRELAND_PBF: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/ireland-and-northern-ireland-latest.osm.pbf"
);

#[derive(Debug, Default)]
pub struct Stats {
    pub min: AtomicUsize,
    pub max: AtomicUsize,
    pub total: AtomicUsize,
    pub count: AtomicUsize,
}

impl Stats {
    pub fn add(&self, value: usize) {
        self.count.fetch_add(1, Ordering::AcqRel);
        self.total.fetch_add(value, Ordering::AcqRel);
        let min = self.min.load(Ordering::Acquire);
        if value < min {
            let _ = self
                .min
                .compare_exchange(min, value, Ordering::AcqRel, Ordering::Relaxed);
        }
        let max = self.max.load(Ordering::Acquire);
        if value > max {
            let _ = self
                .max
                .compare_exchange(max, value, Ordering::AcqRel, Ordering::Relaxed);
        }
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let min = self.min.load(Ordering::Acquire);
        let max = self.max.load(Ordering::Acquire);
        let total = self.total.load(Ordering::Acquire);
        let count = self.count.load(Ordering::Acquire);
        let avg = total as f32 / count as f32;
        write!(
            fmt,
            "avg: {avg}, min: {min}, max: {max}, total: {total}, count: {count}"
        )
    }
}

#[derive(Debug, Default)]
pub struct BBox {
    pub min_lat: AtomicI32,
    pub max_lat: AtomicI32,
    pub min_lon: AtomicI32,
    pub max_lon: AtomicI32,
}

impl BBox {
    pub const VOID: Self = BBox {
        min_lat: AtomicI32::new(1),
        max_lat: AtomicI32::new(-1),
        min_lon: AtomicI32::new(1),
        max_lon: AtomicI32::new(-1),
    };

    pub fn load(&self) -> (i32, i32, i32, i32) {
        (
            self.min_lat.load(Ordering::Acquire),
            self.max_lat.load(Ordering::Acquire),
            self.min_lon.load(Ordering::Acquire),
            self.max_lon.load(Ordering::Acquire),
        )
    }
    pub fn add(&self, lat: i32, lon: i32) {
        self.extend_min_max(lat, lat, lon, lon);
    }
    pub fn extend_min_max(&self, min_lat: i32, max_lat: i32, min_lon: i32, max_lon: i32) {
        if min_lat <= max_lat {
            let min = self.min_lat.load(Ordering::Acquire);
            let max = self.max_lat.load(Ordering::Acquire);
            if min_lat < min || min > max {
                let _ = self.min_lat.compare_exchange(
                    min,
                    min_lat,
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                );
            }
            if max_lat > max || min > max {
                let _ = self.max_lat.compare_exchange(
                    max,
                    max_lat,
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                );
            }
        }
        if min_lon <= max_lon {
            let min = self.min_lon.load(Ordering::Acquire);
            let max = self.max_lon.load(Ordering::Acquire);
            if min_lon < min || min > max {
                let _ = self.min_lon.compare_exchange(
                    min,
                    min_lon,
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                );
            }
            if max_lon > max || min > max {
                let _ = self.max_lon.compare_exchange(
                    max,
                    max_lon,
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                );
            }
        }
    }
    pub fn extend(&self, other: Self) {
        self.extend_min_max(
            other.min_lat.into_inner(),
            other.max_lat.into_inner(),
            other.min_lon.into_inner(),
            other.max_lon.into_inner(),
        );
    }
}

impl fmt::Display for BBox {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let (min_lat, max_lat, min_lon, max_lon) = self.load();
        write!(
            fmt,
            "BBOX(lon:[{min_lon}..{max_lon}], lat:[{min_lat}..{max_lat}])"
        )
    }
}

pub fn bench_file<M: Measurement>(
    group: &mut BenchmarkGroup<'_, M>,
    map_file: &str,
    function: &str,
    bench_fn: fn(&mut File, &Stats, &BBox),
) {
    let size = std::fs::metadata(map_file).unwrap().len();
    let stats = Stats::default();
    let bbox = BBox::VOID;
    group.throughput(Throughput::Bytes(size));
    group.bench_function(function, |b| {
        b.iter_batched_ref(
            || File::open(map_file).unwrap(),
            |file| bench_fn(black_box(file), black_box(&stats), black_box(&bbox)),
            BatchSize::PerIteration,
        )
    });
    println!("BoundingBox: {}", bbox);
    println!("Stats: {}", stats);
}
