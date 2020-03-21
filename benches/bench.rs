use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs::File;
use wasmbin::{WasmbinDecode, WasmbinEncode};

macro_rules! bench_group {
    ($name:ident) => {
        mod $name {
            use super::*;
            use wasmbin::$name::Module;

            fn bench_parse(c: &mut Criterion) {
                c.bench_function(concat!(stringify!($name), "::bench_parse"), |b| {
                    b.iter(|| {
                        let mut f = File::open("temp.wasm").unwrap();
                        Module::decode(&mut f).unwrap()
                    })
                });
            }

            fn bench_parse_buf(c: &mut Criterion) {
                c.bench_function(concat!(stringify!($name), "::bench_parse_buf"), |b| {
                    b.iter(|| {
                        let f = File::open("temp.wasm").unwrap();
                        let mut f = std::io::BufReader::new(f);
                        Module::decode(&mut f).unwrap()
                    })
                });
            }

            fn bench_parse_vec(c: &mut Criterion) {
                let f = std::fs::read("temp.wasm").unwrap();
                c.bench_function(concat!(stringify!($name), "::bench_parse_vec"), |b| {
                    b.iter(|| {
                        let mut f = black_box(f.as_slice());
                        Module::decode(&mut f).unwrap()
                    })
                });
            }

            fn read_module() -> Module {
                let f = std::fs::read("temp.wasm").unwrap();
                let mut f = f.as_slice();
                Module::decode(&mut f).unwrap()
            }

            fn bench_write(c: &mut Criterion) {
                let m = read_module();
                c.bench_function(concat!(stringify!($name), "::bench_write"), |b| {
                    b.iter(|| {
                        let mut f = File::create("temp.out.wasm").unwrap();
                        black_box(&m).encode(&mut f).unwrap();
                        f
                    })
                });
            }

            fn bench_write_buf(c: &mut Criterion) {
                let m = read_module();
                c.bench_function(concat!(stringify!($name), "::bench_write_buf"), |b| {
                    b.iter(|| {
                        let f = File::create("temp.out.wasm").unwrap();
                        let mut f = std::io::BufWriter::new(f);
                        black_box(&m).encode(&mut f).unwrap();
                        f
                    })
                });
            }

            fn bench_write_vec(c: &mut Criterion) {
                let m = read_module();
                c.bench_function(concat!(stringify!($name), "::bench_write_vec"), |b| {
                    b.iter(|| {
                        let mut f = Vec::new();
                        black_box(&m).encode(&mut f).unwrap();
                        f
                    })
                });
            }

            criterion_group! {
                name = benches;
                config = Criterion::default().sample_size(20);
                targets =
                    bench_parse,
                    bench_parse_buf,
                    bench_parse_vec,
                    bench_write,
                    bench_write_buf,
                    bench_write_vec,
            }
        }
    };
}

bench_group!(module);
bench_group!(typed_module);
criterion_main!(module::benches, typed_module::benches);
