use criterion::{
    criterion_group, criterion_main, measurement::Measurement, BenchmarkGroup, Criterion,
};
use osm_pbf_benchmark::{IRELAND_PBF, LICHTENSTEIN_PBF};

pub fn bench_file<M: Measurement>(group: &mut BenchmarkGroup<'_, M>, map_file: &str) {
    osm_pbf_benchmark::impl_osm_pbf_reader::bench_file(group, map_file);
    osm_pbf_benchmark::impl_osm_pbf_reader_pb4::bench_file(group, map_file);
    osm_pbf_benchmark::impl_osmpbf::bench_file(group, map_file);
    osm_pbf_benchmark::impl_osmpbfreader::bench_file(group, map_file);
}

pub fn bench_liechtenstein(c: &mut Criterion) {
    let mut group = c.benchmark_group("liechtenstein");
    bench_file(&mut group, LICHTENSTEIN_PBF);
    group.finish();
}

pub fn bench_ireland(c: &mut Criterion) {
    let mut group = c.benchmark_group("ireland-and-northern-ireland");
    bench_file(&mut group, IRELAND_PBF);
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = bench_liechtenstein, bench_ireland
    //targets = bench_ireland
}
criterion_main!(benches);
