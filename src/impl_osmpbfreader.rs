use crate::{BBox, Stats};
use criterion::{black_box, measurement::Measurement, BenchmarkGroup};
use osmpbfreader::{OsmObj, OsmPbfReader};
use std::{
    fs::File,
    io::BufReader,
    sync::atomic::{AtomicUsize, Ordering},
};

pub const CRATE_NAME: &str = "osmpbfreader";

pub fn decode_sync(pbf_file: &mut File, stats: &Stats, bbox: &BBox) {
    let bufered = BufReader::new(pbf_file);
    let mut reader = OsmPbfReader::new(bufered);
    let mut num_data_blocks = 0;
    for obj in reader.iter() {
        let obj = obj.unwrap();
        match obj {
            OsmObj::Node(ref n) => {
                bbox.add(n.decimicro_lat, n.decimicro_lon);
            }
            _ => {}
        }
        drop(black_box(obj));
        num_data_blocks += 1;
    }
    stats.add(num_data_blocks);
}

pub fn decode_par(pbf_file: &mut File, stats: &Stats, bbox: &BBox) {
    let bufered = BufReader::new(pbf_file);
    let mut reader = OsmPbfReader::new(bufered);
    let num_data_blocks = AtomicUsize::new(0);
    reader.par_iter().for_each(|obj| {
        let obj = obj.unwrap();
        match obj {
            OsmObj::Node(ref n) => {
                bbox.add(n.decimicro_lat, n.decimicro_lon);
            }
            _ => {}
        }
        drop(black_box(obj));
        num_data_blocks.fetch_add(1, Ordering::AcqRel);
    });
    stats.add(num_data_blocks.load(Ordering::Acquire));
}

pub fn bench_file<M: Measurement>(group: &mut BenchmarkGroup<'_, M>, map_file: &str) {
    super::bench_file(group, map_file, "osmpbfreader/decode-sync", decode_sync);
    super::bench_file(group, map_file, "osmpbfreader/decode-par", decode_par);
}
