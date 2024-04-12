use criterion::{black_box, criterion_group, criterion_main, Criterion};
use server::data::list::{CLinkedList, LikeLinkedList};
use std::collections::LinkedList;
use std::sync::RwLock;

fn arc_swap_push(count: usize) {
    let list = CLinkedList::default();

    for x in 0..count {
        black_box(list.push_front(x));
    }
}

fn regular_push(count: usize) {
    let list = RwLock::new(Vec::with_capacity(count));

    for x in 0..count {
        let mut list = black_box(list.write().unwrap());
        black_box(list.push(x));
    }
}

fn regular_ll_push(count: usize) {
    let list = RwLock::new(LinkedList::new());

    for x in 0..count {
        let mut list = black_box(list.write().unwrap());
        black_box(list.push_front(x));
    }
}

fn arc_swap_push_and_pop(count: usize) {
    let list = CLinkedList::default();

    for x in 0..count {
        black_box(list.push_front(x));
    }
    for _ in 0..count {
        black_box(list.pop_back().unwrap());
    }
}

fn regular_push_and_pop(count: usize) {
    let list = RwLock::new(Vec::with_capacity(count));

    for x in black_box(0..count) {
        let mut list = black_box(list.write().unwrap());
        black_box(list.push(x));
    }
    for _ in black_box(0..count) {
        let mut list = black_box(list.write().unwrap());
        black_box(list.pop().unwrap());
    }
}

fn regular_ll_push_and_pop(count: usize) {
    let list = RwLock::new(LinkedList::new());

    for x in black_box(0..count) {
        let mut list = black_box(list.write().unwrap());
        black_box(list.push_front(x));
    }
    for _ in black_box(0..count) {
        let mut list = black_box(list.write().unwrap());
        black_box(list.pop_back().unwrap());
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut custom_group = c.benchmark_group("Arc Swap");

    custom_group.bench_function("Push 10", |b| b.iter(|| black_box(arc_swap_push(10))));
    custom_group.bench_function("Push 100", |b| b.iter(|| black_box(arc_swap_push(100))));
    custom_group.bench_function("Push 1000", |b| b.iter(|| black_box(arc_swap_push(1000))));

    custom_group.bench_function("Push and Pop 10", |b| {
        b.iter(|| black_box(arc_swap_push_and_pop(10)))
    });
    custom_group.bench_function("Push and Pop 100", |b| {
        b.iter(|| black_box(arc_swap_push_and_pop(100)))
    });
    custom_group.bench_function("Push and Pop 1000", |b| {
        b.iter(|| black_box(arc_swap_push_and_pop(1000)))
    });

    drop(custom_group);

    let mut regular_group = c.benchmark_group("Std Vector");

    regular_group.bench_function("Regular Push 10", |b| {
        b.iter(|| black_box(regular_push(10)))
    });
    regular_group.bench_function("Regular Push 100", |b| {
        b.iter(|| black_box(regular_push(100)))
    });
    regular_group.bench_function("Regular Push 1000", |b| {
        b.iter(|| black_box(regular_push(1000)))
    });

    regular_group.bench_function("Regular Push and Pop 10", |b| {
        b.iter(|| black_box(regular_push_and_pop(10)))
    });
    regular_group.bench_function("Regular Push and Pop 100", |b| {
        b.iter(|| black_box(regular_push_and_pop(100)))
    });
    regular_group.bench_function("Regular Push and Pop 1000", |b| {
        b.iter(|| black_box(regular_push_and_pop(1000)))
    });

    drop(regular_group);

    let mut regular_ll_group = c.benchmark_group("Std Linked List");

    regular_ll_group.bench_function("Regular LL Push 10", |b| {
        b.iter(|| black_box(regular_ll_push(10)))
    });
    regular_ll_group.bench_function("Regular LL Push 100", |b| {
        b.iter(|| black_box(regular_ll_push(100)))
    });
    regular_ll_group.bench_function("Regular LL Push 1000", |b| {
        b.iter(|| black_box(regular_ll_push(1000)))
    });

    regular_ll_group.bench_function("Regular LL Push and Pop 10", |b| {
        b.iter(|| black_box(regular_ll_push_and_pop(10)))
    });
    regular_ll_group.bench_function("Regular LL Push and Pop 100", |b| {
        b.iter(|| black_box(regular_ll_push_and_pop(100)))
    });
    regular_ll_group.bench_function("Regular LL Push and Pop 1000", |b| {
        b.iter(|| black_box(regular_ll_push_and_pop(1000)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
