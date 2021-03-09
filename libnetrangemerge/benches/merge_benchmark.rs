use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use libnetrangemerge::{merge_ranges, IpRange, Ipv4Range, RangeInterest};

fn simple_ip_data() -> Vec<RangeInterest<IpRange>> {
    vec![
        RangeInterest::new("127.0.0.0/31".parse().unwrap(), false),
        RangeInterest::new("127.0.0.2/31".parse().unwrap(), true),
        RangeInterest::new("127.0.0.0/30".parse().unwrap(), true),
        RangeInterest::new("127.0.0.4/30".parse().unwrap(), true),
        RangeInterest::new("127.0.0.8/31".parse().unwrap(), true),
        RangeInterest::new("127.0.0.10/31".parse().unwrap(), true),
        RangeInterest::new("127.0.4.0/23".parse().unwrap(), true),
        RangeInterest::new("127.0.6.0/23".parse().unwrap(), true),
    ]
}

fn simple_ipv4_data() -> Vec<RangeInterest<Ipv4Range>> {
    vec![
        RangeInterest::new("127.0.0.0/31".parse().unwrap(), false),
        RangeInterest::new("127.0.0.2/31".parse().unwrap(), true),
        RangeInterest::new("127.0.0.0/30".parse().unwrap(), true),
        RangeInterest::new("127.0.0.4/30".parse().unwrap(), true),
        RangeInterest::new("127.0.0.8/31".parse().unwrap(), true),
        RangeInterest::new("127.0.0.10/31".parse().unwrap(), true),
        RangeInterest::new("127.0.4.0/23".parse().unwrap(), true),
        RangeInterest::new("127.0.6.0/23".parse().unwrap(), true),
    ]
}

fn merge_benchmark(c: &mut Criterion) {
    c.bench_function("merge_benchmark_generic", |b| {
        b.iter_batched(
            || simple_ip_data(),
            |mut ranges| merge_ranges(&mut ranges),
            BatchSize::LargeInput,
        )
    });
    c.bench_function("merge_benchmark_ipv4", |b| {
        b.iter_batched(
            || simple_ipv4_data(),
            |mut ranges| merge_ranges(&mut ranges),
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(benches, merge_benchmark);
criterion_main!(benches);
