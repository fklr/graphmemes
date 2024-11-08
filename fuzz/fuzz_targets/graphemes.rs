#![no_main]
use graphmemes::GraphemeIterator;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _g = GraphemeIterator::new(s, true).collect::<Result<Vec<_>, _>>();
        let _g_no_ansi = GraphemeIterator::new(s, false).collect::<Result<Vec<_>, _>>();
    }
});
