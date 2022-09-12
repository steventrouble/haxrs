use std::sync::{mpsc::Sender, Arc};

use super::{DataTypeEnum, Process};

const MAX_PAGE_BYTES: usize = 1 << 29; // 512 MiB

/// Scans the process for matches.
pub fn scan(
    tx: Sender<usize>,
    process: &Arc<Process>,
    value: &[u8],
    data_type: DataTypeEnum,
    to_filter: &Vec<usize>,
) {
    if to_filter.is_empty() {
        scan_all(tx, process, value)
    } else {
        filter_addresses(tx, process, value, data_type, to_filter)
    }
}

/// Scans the entire process memory for matches.
fn scan_all(tx: Sender<usize>, process: &Arc<Process>, value: &[u8]) {
    let vmem = process.query_vmem();
    for vpage in vmem {
        if vpage.size > MAX_PAGE_BYTES {
            panic!("Partial page scan not supported yet.")
        }
        let mem = process
            .get_mem_at(vpage.start, vpage.size)
            .unwrap_or_else(|_| vec![]);
        scan_page(&tx, &mem, vpage.start, value);
    }
}

/// Scans a single page in memory for matches.
fn scan_page(tx: &Sender<usize>, mem: &Vec<u8>, page_start: usize, value: &[u8]) {
    let len = value.len();
    for (i, v) in mem.chunks_exact(len).enumerate() {
        if v == value {
            tx.send(i * len + page_start).unwrap();
        }
    }
}

/// Filters and existing list of addresses down to only those that match.
fn filter_addresses(
    tx: Sender<usize>,
    process: &Arc<Process>,
    value: &[u8],
    data_type: DataTypeEnum,
    addresses: &Vec<usize>,
) {
    let data_type_size = data_type.info().size_of();
    for addr in addresses {
        let v = process.get_mem_at(*addr, data_type_size);
        if let Ok(v) = v {
            if v == value {
                tx.send(*addr).unwrap();
            }
        }
    }
}
