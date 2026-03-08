use crate::{BBox, Stats};
use criterion::{black_box, measurement::Measurement, BenchmarkGroup};
use osmpbf::{BlobDecode, BlobReader, BlobType, Element, Mmap, MmapBlobReader, PrimitiveBlock};
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{
    fs::File,
    io::BufReader,
    sync::atomic::{AtomicUsize, Ordering},
};

pub const CRATE_NAME: &str = "osmpbf";

fn calc_bbox(primitives: &PrimitiveBlock, num_data_blocks: &AtomicUsize) -> BBox {
    let bbox = BBox::VOID;
    primitives.for_each_element(|e| {
        match e {
            Element::DenseNode(n) => bbox.add(n.decimicro_lat(), n.decimicro_lon()),
            Element::Node(n) => bbox.add(n.decimicro_lat(), n.decimicro_lon()),
            _ => {}
        }
        num_data_blocks.fetch_add(1, Ordering::AcqRel);
    });
    bbox
}

pub fn decode_sync(pbf_file: &mut File, stats: &Stats, bbox: &BBox) {
    let bufered = BufReader::new(pbf_file);
    let reader = BlobReader::new(bufered);
    let num_data_blocks = AtomicUsize::new(0);
    for blob in reader {
        let blob = blob.unwrap();
        if blob.get_type() == BlobType::OsmData {
            let data = blob.to_primitiveblock().unwrap();
            bbox.extend(calc_bbox(&data, &num_data_blocks));
            drop(black_box(data));
        }
    }
    stats.add(num_data_blocks.load(Ordering::Acquire));
}

pub fn decode_par(pbf_file: &mut File, stats: &Stats, bbox: &BBox) {
    let bufered = BufReader::new(pbf_file);
    let reader = BlobReader::new(bufered);
    let num_data_blocks = AtomicUsize::new(0);
    reader.par_bridge().for_each(|blob| {
        let blob = blob.unwrap();
        if blob.get_type() == BlobType::OsmData {
            if let BlobDecode::OsmData(data) = blob.decode().unwrap() {
                bbox.extend(calc_bbox(&data, &num_data_blocks));
                drop(black_box(data)); // TODO
                num_data_blocks.fetch_add(1, Ordering::AcqRel);
            }
        }
    });
    stats.add(num_data_blocks.load(Ordering::Acquire));
}

pub fn decode_par_mmap(pbf_file: &mut File, stats: &Stats, bbox: &BBox) {
    let mmap = unsafe { Mmap::from_file(pbf_file).unwrap() };
    let reader = MmapBlobReader::new(&mmap);
    let num_data_blocks = AtomicUsize::new(0);
    reader.par_bridge().for_each(|blob| {
        let blob = blob.unwrap();
        if blob.get_type() == BlobType::OsmData {
            if let BlobDecode::OsmData(data) = blob.decode().unwrap() {
                bbox.extend(calc_bbox(&data, &num_data_blocks));
                drop(black_box(data)); // TODO
                num_data_blocks.fetch_add(1, Ordering::AcqRel);
            }
        }
    });
    stats.add(num_data_blocks.load(Ordering::Acquire));
}

pub fn bench_file<M: Measurement>(group: &mut BenchmarkGroup<'_, M>, map_file: &str) {
    super::bench_file(group, map_file, "osmpbf/decode-sync", decode_sync);
    super::bench_file(group, map_file, "osmpbf/decode-par", decode_par);
    super::bench_file(group, map_file, "osmpbf/decode-par-mmap", decode_par);
}
