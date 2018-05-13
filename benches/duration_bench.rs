#[macro_use]
extern crate bencher;
extern crate time_parse;

use bencher::Bencher;
use bencher::black_box;

fn hand(bench: &mut Bencher) {
    bench.iter(|| {
        time_parse::duration::parse(black_box("P1DT7M7S"))
    })
}

fn nom(bench: &mut Bencher) {
    bench.iter(|| {
        time_parse::duration::parse_nom(black_box("P1DT7M7S"))
    });
}

benchmark_group!(benches, hand, nom);
benchmark_main!(benches);
