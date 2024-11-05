use graphmemes::GraphemeIterator;
use owo_colors::OwoColorize;
use std::{
    hint::black_box,
    time::{Duration, Instant},
};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
struct PerformanceMetrics {
    duration: Duration,
    iterations_per_sec: f64,
    bytes_per_sec: f64,
}

fn measure_performance<F>(input: &str, iterations: usize, f: F) -> PerformanceMetrics
where
    F: Fn(&str) -> usize,
{
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(f(input));
    }
    let duration = start.elapsed();

    let secs = duration.as_secs_f64();
    PerformanceMetrics {
        duration,
        iterations_per_sec: iterations as f64 / secs,
        bytes_per_sec: (input.len() * iterations) as f64 / secs,
    }
}

// Function using String allocation for comparison
fn with_alloc(input: &str, count_ansi: bool) -> usize {
    if input.is_empty() {
        return 0;
    }

    let mut count = 0;
    let mut text_buffer = String::new();
    let mut in_ansi = false;

    for c in input.chars() {
        match (c, in_ansi) {
            ('\x1b', _) => {
                if !text_buffer.is_empty() {
                    count += text_buffer.graphemes(true).count();
                    text_buffer.clear();
                }
                in_ansi = true;
            }
            (_, true) => {
                if c.is_ascii_alphabetic() {
                    in_ansi = false;
                    if count_ansi {
                        count += 1;
                    }
                }
            }
            (c, false) => text_buffer.push(c),
        }
    }

    if !text_buffer.is_empty() {
        count += text_buffer.graphemes(true).count();
    }

    count
}

fn main() {
    println!("{}", "\nPerformance Pattern Analysis".bold().cyan());
    println!("{}", "==========================".cyan());

    // Pre-allocate test strings
    let ascii = "Hello, world!".to_string();
    let unicode = "Hello 👋 World!".to_string();
    let ansi = "\x1b[31mHello\x1b[0m".to_string();
    let mixed = "Hello 👋 \x1b[31mWorld\x1b[0m!".to_string();
    let long = "Hello, world! ".repeat(100);
    let complex = "👨‍👩‍👧‍👦\x1b[31mTest\x1b[0m".to_string();

    let test_cases = vec![
        ("ascii", &ascii, 10000),
        ("unicode", &unicode, 10000),
        ("ansi", &ansi, 10000),
        ("mixed", &mixed, 10000),
        ("long", &long, 1000),
        ("complex", &complex, 10000),
    ];

    for (name, input, iterations) in test_cases {
        println!("\n{}: {}", "Test Case".cyan(), name);

        let alloc_metrics = measure_performance(input, iterations, |s| with_alloc(s, false));
        let zero_metrics = measure_performance(input, iterations, |s| {
            GraphemeIterator::new(s, false).count()
        });

        println!("  {}", "Allocating Version:".bold());
        println!("    Time: {:?}", alloc_metrics.duration);
        println!(
            "    Throughput: {:.2} iter/sec",
            alloc_metrics.iterations_per_sec
        );
        println!(
            "    Bandwidth: {:.2} MB/sec",
            alloc_metrics.bytes_per_sec / 1_000_000.0
        );

        println!("  {}", "Zero-Alloc Version:".bold());
        println!("    Time: {:?}", zero_metrics.duration);
        println!(
            "    Throughput: {:.2} iter/sec",
            zero_metrics.iterations_per_sec
        );
        println!(
            "    Bandwidth: {:.2} MB/sec",
            zero_metrics.bytes_per_sec / 1_000_000.0
        );

        // Performance comparison
        let speedup =
            alloc_metrics.duration.as_nanos() as f64 / zero_metrics.duration.as_nanos() as f64;
        println!(
            "  {}: {:.2}{}",
            "Speedup".green(),
            speedup.bold(),
            "x".bold()
        );
    }
}
