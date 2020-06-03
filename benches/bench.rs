use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs::File;
use wasmbin::Module;

fn deep_module() -> Module {
    use wasmbin::builtins::Blob;
    use wasmbin::instructions::{Expression, Instruction};
    use wasmbin::sections::FuncBody;
    use wasmbin::types::BlockType;

    let mut expr = Expression::default();
    for _ in 0..100_000 {
        expr.push(Instruction::BlockStart(BlockType::Empty));
    }
    for _ in 0..100_000 {
        expr.push(Instruction::End);
    }
    let raw = Module {
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
            let f = File::open("temp.wasm").unwrap();
            Module::decode_from(f).unwrap()
        })
    });
}

fn bench_parse_buf(c: &mut Criterion) {
    c.bench_function(concat!(stringify!($name), "::bench_parse_buf"), |b| {
        b.iter(|| {
            let f = File::open("temp.wasm").unwrap();
            let f = std::io::BufReader::new(f);
            Module::decode_from(f).unwrap()
        })
    });
}

fn bench_parse_vec(c: &mut Criterion) {
    c.bench_function(concat!(stringify!($name), "::bench_parse_vec"), |b| {
        let f = std::fs::read("temp.wasm").unwrap();
        b.iter(|| {
            let f = black_box(f.as_slice());
            Module::decode_from(f).unwrap()
        })
    });
}

fn bench_parse_deep_module(c: &mut Criterion) {
    c.bench_function(
        concat!(stringify!($name), "::bench_parse_deep_module"),
        |b| {
            assert!(cfg!(not(feature = "lazy-blob")));
            let f = deep_module().encode_into(Vec::new()).unwrap();
            b.iter(|| {
                let f = black_box(f.as_slice());
                Module::decode_from(f).unwrap()
            })
        },
    );
}

fn read_module() -> Module {
    let f = std::fs::read("temp.wasm").unwrap();
    Module::decode_from(f.as_slice()).unwrap()
}

fn bench_write(c: &mut Criterion) {
    c.bench_function(concat!(stringify!($name), "::bench_write"), |b| {
        let m = read_module();
        b.iter(|| {
            let f = File::create("temp.out.wasm").unwrap();
            black_box(&m).encode_into(f).unwrap()
        })
    });
}

fn bench_write_buf(c: &mut Criterion) {
    c.bench_function(concat!(stringify!($name), "::bench_write_buf"), |b| {
        let m = read_module();
        b.iter(|| {
            let f = File::create("temp.out.wasm").unwrap();
            let f = std::io::BufWriter::new(f);
            black_box(&m).encode_into(f).unwrap()
        })
    });
}

fn bench_write_vec(c: &mut Criterion) {
    c.bench_function(concat!(stringify!($name), "::bench_write_vec"), |b| {
        let m = read_module();
        b.iter(|| black_box(&m).encode_into(Vec::new()).unwrap())
    });
}

fn bench_write_deep_module(c: &mut Criterion) {
    c.bench_function(
        concat!(stringify!($name), "::bench_write_deep_module"),
        |b| {
            assert!(cfg!(not(feature = "lazy-blob")));
            let m = deep_module();
            b.iter(|| black_box(&m).encode_into(Vec::new()).unwrap())
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

criterion_main!(benches);
