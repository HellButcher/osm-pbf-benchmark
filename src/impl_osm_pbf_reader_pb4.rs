use crate::{BBox, Stats};
use criterion::{black_box, measurement::Measurement, BenchmarkGroup};
use osm_pbf_reader_pb4::{
    data::{primitives::*, PrimitiveBlock},
    Blobs,
};
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{
    fs::File,
    sync::atomic::{AtomicUsize, Ordering},
};

pub const CRATE_NAME: &str = "osm-pbf-reader-pb4";

fn calc_bbox(primitives: &PrimitiveBlock, num_data_blocks: &AtomicUsize) -> BBox {
    let bbox = BBox::VOID;
    for p in primitives.iter_elements() {
        if let Element::Node(n) = p {
            bbox.add((n.lat_nano() / 100) as i32, (n.lon_nano() / 100) as i32);
        }
        num_data_blocks.fetch_add(1, Ordering::AcqRel);
    }
    bbox
}

pub fn decode_sync(pbf_file: &mut File, stats: &Stats, bbox: &BBox) {
    let blobs = Blobs::from_read(pbf_file).unwrap();
    let num_data_blocks = AtomicUsize::new(0);
    for blob in blobs {
        let data = blob.unwrap().into_decoded().unwrap();
        bbox.extend(calc_bbox(&data, &num_data_blocks));
        drop(black_box(data)); // TODO
    }
    stats.add(num_data_blocks.load(Ordering::Acquire));
}

pub fn decode_sync2(pbf_file: &mut File, stats: &Stats, bbox: &BBox) {
    let mut blobs = Blobs::from_read(pbf_file).unwrap();
    let num_data_blocks = AtomicUsize::new(0);
    while let Some(data) = blobs.next_primitive_block_decoded().unwrap() {
        bbox.extend(calc_bbox(&data, &num_data_blocks));
        drop(black_box(data)); // TODO
    }
    stats.add(num_data_blocks.load(Ordering::Acquire));
}

pub fn decode_par(pbf_file: &mut File, stats: &Stats, bbox: &BBox) {
    let blobs = Blobs::from_read(pbf_file).unwrap();
    let num_data_blocks = AtomicUsize::new(0);
    blobs.par_bridge().for_each(|blob| {
        let data = blob.unwrap().into_decoded().unwrap();
        bbox.extend(calc_bbox(&data, &num_data_blocks));
        drop(black_box(data));
    });
    stats.add(num_data_blocks.load(Ordering::Acquire));
}

pub fn bench_file<M: Measurement>(group: &mut BenchmarkGroup<'_, M>, map_file: &str) {
    super::bench_file(
        group,
        map_file,
        "osm-pbf-reader-pb4/decode-sync",
        decode_sync,
    );
    super::bench_file(
        group,
        map_file,
        "osm-pbf-reader-pb4/decode-sync2",
        decode_sync2,
    );
    super::bench_file(group, map_file, "osm-pbf-reader-pb4/decode-par", decode_par);
}
