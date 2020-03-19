#![feature(test)]

extern crate test;

use std::fs::File;
use test::{black_box, Bencher};
use wasmbin::{module::Module, WasmbinDecode, WasmbinEncode};

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
fn bench_parse_mmap(b: &mut Bencher) {
    b.iter(|| {
        let f = File::open("temp.wasm").unwrap();
        let mut f = &*unsafe { memmap::Mmap::map(&f) }.unwrap();
        let m = Module::decode(&mut f).unwrap();
        black_box(m)
    })
}

#[bench]
fn bench_write(b: &mut Bencher) {
    let m = {
        let f = File::open("temp.wasm").unwrap();
        let mut f = &*unsafe { memmap::Mmap::map(&f) }.unwrap();
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
        let f = File::open("temp.wasm").unwrap();
        let mut f = &*unsafe { memmap::Mmap::map(&f) }.unwrap();
        Module::decode(&mut f).unwrap()
    };
    b.iter(|| {
        let f = File::create("temp.out.wasm").unwrap();
        let mut f = std::io::BufWriter::new(f);
        black_box(&m).encode(&mut f).unwrap();
    })
}
