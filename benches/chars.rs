use graphmemes::GraphemeIterator;
use owo_colors::OwoColorize;
use std::{fmt::Display, fs, time::Instant};
use unicode_segmentation::UnicodeSegmentation;

const FILES: &[&str] = &[
    "arabic",
    "english",
    "hindi",
    "japanese",
    "korean",
    "mandarin",
    "russian",
    "source_code",
];

#[inline(always)]
fn grapheme(text: &str) {
    for c in UnicodeSegmentation::graphemes(black_box(text), true) {
        black_box(c);
    }
}

#[inline(always)]
fn graphmemes_grapheme(text: &str) {
    for c in GraphemeIterator::new(black_box(text), false) {
        let _ = black_box(c);
    }
}

#[inline(always)]
fn scalar(text: &str) {
    for c in black_box(text).chars() {
        black_box(c);
    }
}

fn faster() -> Box<dyn Display> {
    Box::new("faster".bright_green())
}

fn slower() -> Box<dyn Display> {
    Box::new("slower".bright_red())
}

fn test_performance(name: &str, content: &str) {
    let iterations = 20000;

    // Warmup
    for _ in 0..5 {
        grapheme(content);
        graphmemes_grapheme(content);
        scalar(content);
    }

    println!("\n{}", "Test Case:".cyan().bold());
    println!(
        "{:<12} size: {}, iter: {}",
        name.yellow(),
        content.len().to_string().bright_green(),
        iterations.to_string().bright_green()
    );
    println!("{}", "=".repeat(50).cyan());

    let start = Instant::now();
    for _ in 0..iterations {
        grapheme(content);
    }
    let uni_time = start.elapsed();

    let start = Instant::now();
    for _ in 0..iterations {
        graphmemes_grapheme(content);
    }
    let graphmemes_time = start.elapsed();

    let start = Instant::now();
    for _ in 0..iterations {
        scalar(content);
    }
    let scalar_time = start.elapsed();

    println!("{}", "Performance:".cyan().bold());
    println!(
        "unicode-seg:  {}",
        format!("{:.3?}", uni_time).bright_blue()
    );
    println!(
        "graphmemes:     {}",
        format!("{:.3?}", graphmemes_time).bright_blue()
    );
    println!(
        "scalar:       {}",
        format!("{:.3?}", scalar_time).bright_blue()
    );

    println!("\n{}", "Relative Performance:".cyan().bold());
    let uni_ratio = uni_time.as_secs_f64() / graphmemes_time.as_secs_f64();
    let scalar_ratio = graphmemes_time.as_secs_f64() / scalar_time.as_secs_f64();

    let speed_indicator = match uni_ratio {
        x if x > 1.0 => faster(),
        _ => slower(),
    };

    println!(
        "vs unicode:   {}x {}",
        format!("{:.2}", uni_ratio).bright_yellow().bold(),
        speed_indicator
    );

    println!(
        "vs scalar:    {:.2}x {}",
        scalar_ratio.bright_yellow().bold(),
        slower().bright_green()
    );
}

fn main() {
    println!(
        "\n{}",
        "Grapheme Cluster Iterator Performance".bold().cyan()
    );
    println!("{}", "==================================".cyan());

    for file in FILES {
        let content = fs::read_to_string(format!("benches/texts/{}.txt", file))
            .unwrap_or_else(|_| panic!("Could not read file: {}", file));
        test_performance(file, &content);
    }
}

#[inline(never)]
fn black_box<T>(dummy: T) -> T {
    std::hint::black_box(dummy)
}
