#![feature(new_zeroed_alloc)]
#![feature(maybe_uninit_slice)]
#![feature(read_buf)]
#![feature(core_io_borrowed_buf)]

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::fs::File;
use std::io::{BorrowedBuf, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::FileExt;

const READ_SIZE: usize = 1024 * 1024;

fn prepare_file() -> File {
    let path = "/tmp/readchunks.data";
    let mut file = File::create(path).unwrap();
    let chunk = (0..1024).map(|i| (i % 256) as u8).collect::<Vec<_>>();
    for _ in 0..READ_SIZE.div_ceil(chunk.len()) {
        file.write_all(chunk.as_slice()).unwrap();
    }
    file.flush().unwrap();

    File::open(path).unwrap()
}

#[inline(never)]
fn read_exact_alloc_zeroed(file: &File) {
    let mut buf = Box::new_zeroed_slice(READ_SIZE);
    // Safety: zero is a valid value for u8
    let buf = unsafe { buf.assume_init_mut() };
    file.read_exact_at(buf, 0).unwrap();
    assert_eq!(buf.len(), READ_SIZE);
}

#[inline(never)]
fn read_exact_alloc_uninit(file: &File) {
    let mut buf = Box::new_uninit_slice(READ_SIZE);
    // Safety: Read impl for File does not read the uninitialized bytes in buf
    let buf = unsafe { buf.assume_init_mut() };
    file.read_exact_at(buf, 0).unwrap();
    assert_eq!(buf.len(), READ_SIZE);
}

#[inline(never)]
fn read_exact_zeroed(file: &File, buf: &mut Vec<u8>) {
    buf.clear();
    buf.resize(READ_SIZE, 0);
    file.read_exact_at(buf, 0).unwrap();
    assert_eq!(buf.len(), READ_SIZE);
}

#[inline(never)]
fn read_exact_uninit(file: &File, buf: &mut Vec<u8>) {
    buf.clear();
    buf.reserve(READ_SIZE);
    // Safety: Read impl for File does not read the buffer
    unsafe {
        buf.set_len(READ_SIZE);
    }
    file.read_exact_at(buf, 0).unwrap();
    assert_eq!(buf.len(), READ_SIZE);
}

#[inline(never)]
fn read_take_to_end(file: &mut File, buf: &mut Vec<u8>) {
    buf.clear();
    buf.reserve(READ_SIZE);
    file.seek(SeekFrom::Start(0)).unwrap();
    file.take(READ_SIZE as _).read_to_end(buf).unwrap();
    assert_eq!(buf.len(), READ_SIZE);
}

#[inline(never)]
fn read_buf(file: &mut File, buf: &mut Vec<u8>) {
    buf.clear();
    buf.reserve(READ_SIZE);
    let mut borrowed_buf = BorrowedBuf::from(&mut buf.spare_capacity_mut()[..READ_SIZE]);
    file.seek(SeekFrom::Start(0)).unwrap();
    file.read_buf_exact(borrowed_buf.unfilled()).unwrap();
    assert_eq!(borrowed_buf.len(), READ_SIZE);
}

pub fn bench_readchunks(c: &mut Criterion) {
    let mut file = prepare_file();
    let file_size = file.metadata().unwrap().len();
    println!("file size: {file_size}");

    assert!((READ_SIZE as u64) <= file_size);

    {
        // warmup
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();
    }

    let mut buffer = Vec::new();

    let mut group = c.benchmark_group("readchunks");
    group.throughput(Throughput::Bytes(READ_SIZE as u64));
    group.bench_function("read_exact_alloc_zeroed", |b| {
        b.iter(|| {
            read_exact_alloc_zeroed(&mut file);
        })
    });
    group.bench_function("read_exact_alloc_uninit", |b| {
        b.iter(|| {
            read_exact_alloc_uninit(&mut file);
        })
    });
    group.bench_function("read_exact_zeroed", |b| {
        b.iter(|| {
            read_exact_zeroed(&mut file, &mut buffer);
        })
    });
    group.bench_function("read_exact_uninit", |b| {
        b.iter(|| {
            read_exact_uninit(&mut file, &mut buffer);
        })
    });
    group.bench_function("read_take_to_end", |b| {
        b.iter(|| {
            read_take_to_end(&mut file, &mut buffer);
        })
    });
    group.bench_function("read_buf", |b| {
        b.iter(|| {
            read_buf(&mut file, &mut buffer);
        })
    });
}

criterion_group!(benches, bench_readchunks);
criterion_main!(benches);
