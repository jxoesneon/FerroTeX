use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use ferrotex_syntax::parse;

fn generate_synthetic_tex(lines: usize) -> String {
    let mut s = String::with_capacity(lines * 50);
    s.push_str("\\documentclass{article}\n\\begin{document}\n");
    for i in 0..lines {
        s.push_str(&format!("\\section{{Section {}}}\n", i));
        s.push_str("Here is some text with \\textbf{bold} and \\textit{italic}.\n");
        s.push_str("Some math: $E = mc^2$ and \\[ \\int_0^\\infty e^{-x} dx \\].\n");
        s.push_str("\\begin{itemize}\n");
        s.push_str("  \\item Item 1\n");
        s.push_str("  \\item Item 2\n");
        s.push_str("\\end{itemize}\n");
    }
    s.push_str("\\end{document}\n");
    s
}

fn benchmark_parser(c: &mut Criterion) {
    let small_input = generate_synthetic_tex(10);
    let medium_input = generate_synthetic_tex(100);
    let large_input = generate_synthetic_tex(1000);

    let mut group = c.benchmark_group("parser");

    group.bench_function("parse_small_10_sections", |b| {
        b.iter(|| parse(black_box(&small_input)))
    });

    group.bench_function("parse_medium_100_sections", |b| {
        b.iter(|| parse(black_box(&medium_input)))
    });

    // This is the stress test target
    group.bench_function("parse_large_1000_sections", |b| {
        b.iter(|| parse(black_box(&large_input)))
    });

    group.finish();
}

criterion_group!(benches, benchmark_parser);
criterion_main!(benches);
