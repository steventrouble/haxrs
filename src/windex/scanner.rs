use std::sync::Arc;

use super::{DataTypeTrait, Process};

const MAX_PAGE_BYTES: usize = 1 << 29; // 512 MiB
const MIN_RESULTS_CAPACITY: usize = 1 << 14; // 16 KiB

/// Scans the entire process for a match.
pub fn scan(
    results: &mut Vec<usize>,
    process: &Arc<Process>,
    value: &[u8],
    data_type: Box<dyn DataTypeTrait>,
) {
    if results.capacity() < MIN_RESULTS_CAPACITY {
        results.reserve(MIN_RESULTS_CAPACITY);
    }

    if results.is_empty() {
        scan_all(results, process, value)
    } else {
        filter_addresses(results, process, value, data_type)
    }
}

fn scan_all(results: &mut Vec<usize>, process: &Arc<Process>, value: &[u8]) {
    let vmem = process.query_vmem();
    for vpage in vmem {
        if vpage.size > MAX_PAGE_BYTES {
            panic!("Partial page scan not supported yet.")
        }
        let mem = process
            .get_mem_at(vpage.start, vpage.size)
            .unwrap_or_else(|_| vec![]);
        scan_page(results, &mem, vpage.start, value);
    }
}

fn scan_page(results: &mut Vec<usize>, mem: &Vec<u8>, page_start: usize, value: &[u8]) {
    let len = value.len();
    for (i, v) in mem.chunks_exact(len).enumerate() {
        if v == value {
            results.push(i * len + page_start);
        }
    }
}

fn filter_addresses(
    addresses: &mut Vec<usize>,
    process: &Arc<Process>,
    value: &[u8],
    data_type: Box<dyn DataTypeTrait>,
) {
    addresses.retain(|addr| {
        let v = process.get_mem_at(*addr, data_type.size_of());
        match v {
            Ok(v) => v == value,
            Err(_) => false,
        }
    })
}
