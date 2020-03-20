#![feature(test)]

extern crate test;

use std::fs::File;
use test::{black_box, Bencher};
use wasmbin::{WasmbinDecode, WasmbinEncode};

macro_rules! bench_group {
    ($name:ident) => {
        mod $name {
            use super::*;
            use wasmbin::$name::Module;

            #[bench]
            fn bench_parse(b: &mut Bencher) {
                b.iter(|| {
                    let mut f = File::open("temp.wasm").unwrap();
                    let m = Module::decode(&mut f).unwrap();
                    black_box(m)
                })
            }

            #[bench]
            fn bench_parse_buf(b: &mut Bencher) {
                b.iter(|| {
                    let f = File::open("temp.wasm").unwrap();
                    let mut f = std::io::BufReader::new(f);
                    let m = Module::decode(&mut f).unwrap();
                    black_box(m)
                })
            }

            #[bench]
            fn bench_parse_vec(b: &mut Bencher) {
                let f = std::fs::read("temp.wasm").unwrap();
                b.iter(|| {
                    let mut f = f.as_slice();
                    let m = Module::decode(&mut f).unwrap();
                    black_box(m)
                })
            }

            #[bench]
            fn bench_write(b: &mut Bencher) {
                let m = {
                    let f = std::fs::read("temp.wasm").unwrap();
                    let mut f = f.as_slice();
                    Module::decode(&mut f).unwrap()
                };
                b.iter(|| {
                    let mut f = File::create("temp.out.wasm").unwrap();
                    black_box(&m).encode(&mut f).unwrap();
                })
            }

            #[bench]
            fn bench_write_buf(b: &mut Bencher) {
                let m = {
                    let f = std::fs::read("temp.wasm").unwrap();
                    let mut f = f.as_slice();
                    Module::decode(&mut f).unwrap()
                };
                b.iter(|| {
                    let f = File::create("temp.out.wasm").unwrap();
                    let mut f = std::io::BufWriter::new(f);
                    black_box(&m).encode(&mut f).unwrap();
                })
            }
        }
    };
}

bench_group!(module);
bench_group!(typed_module);
