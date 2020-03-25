use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs::File;
use wasmbin::io::{WasmbinDecode, WasmbinEncode};

macro_rules! bench_group {
    ($namespace:ident as $name:ident) => {
        mod $name {
            use super::*;
            use wasmbin::$namespace::Module;

            fn deep_module() -> Module {
                use wasmbin::builtins::Blob;
                use wasmbin::instructions::{BlockBody, Expression, Instruction};
                use wasmbin::sections::FuncBody;
                use wasmbin::types::BlockType;

                let mut expr = Expression::default();
                for _ in 0..100_000 {
                    let old_expr = std::mem::take(&mut expr);
                    expr.push(Instruction::Block(BlockBody {
                        return_type: BlockType::Empty,
                        expr: old_expr,
                    }));
                }
                let raw = wasmbin::module::Module {
                    sections: vec![vec![Blob::from(FuncBody {
                        locals: Default::default(),
                        expr,
                    })]
                    .into()],
                    ..Default::default()
                };
                std::convert::TryFrom::try_from(raw).unwrap()
            }

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
                c.bench_function(concat!(stringify!($name), "::bench_parse_vec"), |b| {
                    let f = std::fs::read("temp.wasm").unwrap();
                    b.iter(|| {
                        let mut f = black_box(f.as_slice());
                        Module::decode(&mut f).unwrap()
                    })
                });
            }

            fn bench_parse_deep_module(c: &mut Criterion) {
                c.bench_function(
                    concat!(stringify!($name), "::bench_parse_deep_module"),
                    |b| {
                        assert!(cfg!(not(feature = "lazy-blob")));
                        let mut f = Vec::new();
                        deep_module().encode(&mut f).unwrap();
                        b.iter(|| {
                            let mut f = black_box(f.as_slice());
                            Module::decode(&mut f).unwrap()
                        })
                    },
                );
            }

            fn read_module() -> Module {
                let f = std::fs::read("temp.wasm").unwrap();
                let mut f = f.as_slice();
                Module::decode(&mut f).unwrap()
            }

            fn bench_write(c: &mut Criterion) {
                c.bench_function(concat!(stringify!($name), "::bench_write"), |b| {
                    let m = read_module();
                    b.iter(|| {
                        let mut f = File::create("temp.out.wasm").unwrap();
                        black_box(&m).encode(&mut f).unwrap();
                        f
                    })
                });
            }

            fn bench_write_buf(c: &mut Criterion) {
                c.bench_function(concat!(stringify!($name), "::bench_write_buf"), |b| {
                    let m = read_module();
                    b.iter(|| {
                        let f = File::create("temp.out.wasm").unwrap();
                        let mut f = std::io::BufWriter::new(f);
                        black_box(&m).encode(&mut f).unwrap();
                        f
                    })
                });
            }

            fn bench_write_vec(c: &mut Criterion) {
                c.bench_function(concat!(stringify!($name), "::bench_write_vec"), |b| {
                    let m = read_module();
                    b.iter(|| {
                        let mut f = Vec::new();
                        black_box(&m).encode(&mut f).unwrap();
                        f
                    })
                });
            }

            fn bench_write_deep_module(c: &mut Criterion) {
                c.bench_function(
                    concat!(stringify!($name), "::bench_write_deep_module"),
                    |b| {
                        assert!(cfg!(not(feature = "lazy-blob")));
                        let m = deep_module();
                        b.iter(|| {
                            let mut f = Vec::new();
                            black_box(&m).encode(&mut f).unwrap();
                            f
                        })
                    },
                );
            }

            criterion_group! {
                name = benches;
                config = Criterion::default().sample_size(20);
                targets =
                    bench_parse,
                    bench_parse_buf,
                    bench_parse_vec,
                    bench_parse_deep_module,
                    bench_write,
                    bench_write_buf,
                    bench_write_vec,
                    bench_write_deep_module,
            }
        }
    };
}

bench_group!(module as raw);
bench_group!(typed_module as typed);
criterion_main!(raw::benches, typed::benches);
