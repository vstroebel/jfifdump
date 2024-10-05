#![no_main]

use jfifdump::{Reader, SegmentKind};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut reader = match Reader::new(data) {
        Ok(r) => r,
        Err(_) => return,
    };

    loop {
        let segment = match reader.next_segment() {
            Ok(s) => s,
            Err(_) => return,
        };

        match segment.kind {
            SegmentKind::Eoi => break,
            SegmentKind::Frame(frame) => {
                println!("{}x{}", frame.dimension_x, frame.dimension_y);
                break;
            }
            _ => {}
        }
    }
});
