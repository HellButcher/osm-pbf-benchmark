# Benchmarks for osm-pbf-reader and osm-pbf-proto

Download `.osm.pbf` files first, and place them in the root of the checked out repository.

- [ireland-and-northern-ireland-latest.osm.pbf](https://download.geofabrik.de/europe/ireland-and-northern-ireland.html)
- [liechtenstein-latest.osm.pbf](https://download.geofabrik.de/europe/liechtenstein.html)

Run the benchmarks with 

```bash
cargo bench
```

See the output reports in `target/criterion/reports/index.html`

See also [Criterion.rs](https://bheisler.github.io/criterion.rs/book/)

