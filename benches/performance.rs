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

fn generate_stress_test_data() -> Vec<(String, String)> {
    let mut test_cases = Vec::new();

    // Base stress tests

    // 1. Long ASCII text (100KB)
    test_cases.push((
        "ascii_long".to_string(),
        "The quick brown fox jumps over the lazy dog. ".repeat(2048),
    ));

    // 2. Mixed Unicode text (50KB)
    let unicode_sample = "Hello 👋 世界! Café école garçon über niño año. ";
    test_cases.push(("unicode_mixed".to_string(), unicode_sample.repeat(1024)));

    // 3. Heavy ANSI formatting (30KB)
    let ansi_sample = "\x1b[31mred\x1b[0m \x1b[32mgreen\x1b[0m \x1b[34mblue\x1b[0m ";
    test_cases.push(("ansi_heavy".to_string(), ansi_sample.repeat(1024)));

    // 4. Complex emoji sequences (20KB)
    let emoji_sample = "👨‍👩‍👧‍👦 👩🏽‍💻 🇺🇸 🌈 ";
    test_cases.push(("emoji_complex".to_string(), emoji_sample.repeat(512)));

    // 5. RTL with combining marks (15KB)
    let rtl_sample = "مرحباً بالعالم العربي كيف حالك؟ ";
    test_cases.push(("rtl_combining".to_string(), rtl_sample.repeat(256)));

    // 6. Mixed stress test (combines all) (~40KB)
    let mixed = format!(
        "{}{}{}{}{}",
        "Plain ASCII | ",
        "Unicode 世界 | ",
        "\x1b[35mANSI\x1b[0m | ",
        "Emoji 👨‍👩‍👧‍👦 | ",
        "RTL مرحباً | "
    );
    test_cases.push(("mixed_stress".to_string(), mixed.repeat(1024)));

    // Extended stress tests

    // 7. Nested ANSI sequences (40KB)
    let nested_ansi =
        "\x1b[1m\x1b[31mBold Red\x1b[0m\x1b[0m \x1b[4m\x1b[32mUnderline Green\x1b[0m\x1b[0m ";
    test_cases.push(("nested_ansi".to_string(), nested_ansi.repeat(512)));

    // 8. Complex combining marks (30KB)
    let combining = "a\u{0301}\u{0302}\u{0303} e\u{0301}\u{0304} i\u{0302}\u{0300} o\u{0303}\u{0301} u\u{0304}\u{0302} ";
    test_cases.push(("heavy_combining".to_string(), combining.repeat(1024)));

    // 9. Emoji with skin tones and ZWJ (25KB)
    let complex_emoji = "👨🏽‍💻 👩🏾‍🏫 👨🏿‍🌾 👩🏻‍🔬 👨🏼‍🎨 ";
    test_cases.push(("emoji_skin_tones".to_string(), complex_emoji.repeat(512)));

    // 10. Mixed scripts (35KB)
    let mixed_scripts = "Hello མཐའི་རྒྱ עברית العربية 한글 ไทย ";
    test_cases.push(("mixed_scripts".to_string(), mixed_scripts.repeat(1024)));

    // 11. Boundary edge cases (20KB)
    let boundaries = "a\u{200D}b👨‍👩\u{200D}👧\u{200D}👦क्षि ";
    test_cases.push(("boundary_cases".to_string(), boundaries.repeat(512)));

    // 12. Heavy ANSI with text breaks (45KB)
    let ansi_breaks = "\x1b[31mRed\x1b[0m\n\x1b[32mGreen\x1b[0m\t\x1b[34mBlue\x1b[0m ";
    test_cases.push(("ansi_with_breaks".to_string(), ansi_breaks.repeat(1024)));

    // 13. Large mixed content document (150KB)
    let document = format!(
        "{}\n{}\n{}\n{}\n{}\n",
        "# \x1b[1mDocument Title\x1b[0m",
        "Hello 世界! Here's some *formatted* text with 👋 emoji.",
        "## \x1b[32mSection 1\x1b[0m\nمرحباً بالعالم",
        "Some code: `let x = 42;` and more 👨‍💻 content",
        "### \x1b[34mConclusion\x1b[0m\nThe End! 🎉"
    );
    test_cases.push(("large_document".to_string(), document.repeat(512)));

    // 14. Pathological ANSI case (30KB)
    let pathological_ansi =
        "\x1b[31m\x1b[1m\x1b[4m\x1b[5m\x1b[7mTest\x1b[0m\x1b[0m\x1b[0m\x1b[0m\x1b[0m ";
    test_cases.push((
        "pathological_ansi".to_string(),
        pathological_ansi.repeat(512),
    ));

    // 15. Extended grapheme clusters (25KB)
    let extended_graphemes = "a\u{0301}\u{0302}b\u{0301}\u{0303}c\u{0304}\u{0305}नमस्ते ";
    test_cases.push((
        "extended_graphemes".to_string(),
        extended_graphemes.repeat(512),
    ));

    // 16. Mixed length content (40KB)
    let mixed_length = format!(
        "{}{}{}{}",
        "Short ",
        "Medium length text ",
        "Much longer piece of text that extends quite a bit further ",
        "Really long text that goes on and on with some emoji 👨‍👩‍👧‍👦 and formatting \x1b[31min red\x1b[0m "
    );
    test_cases.push(("mixed_length".to_string(), mixed_length.repeat(256)));

    test_cases
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
        run_performance_test(input, iterations);
    }

    println!("\n{}", "Extended Stress Test Analysis".bold().magenta());
    println!("{}", "============================".magenta());

    let stress_cases = generate_stress_test_data();
    for (name, input) in stress_cases {
        println!("\n{}: {}", "Stress Test".magenta(), name);
        // Lower iterations for stress tests due to larger input size
        run_performance_test(&input, 100);
    }
}

fn run_performance_test(input: &str, iterations: usize) {
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

    // Memory usage note for large inputs
    if input.len() > 10000 {
        println!(
            "  {}: {:.2} MB",
            "Input Size".yellow(),
            input.len() as f64 / 1_000_000.0
        );
    }
}
