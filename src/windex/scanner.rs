use std::sync::{mpsc::Sender, Arc};

use super::{DataTypeEnum, Process};

const MAX_PAGE_BYTES: usize = 1 << 29; // 512 MiB

#[derive(Clone)]
pub struct SearchResult {
    pub address: usize,
    pub data_type: DataTypeEnum,
    pub value: Vec<u8>,
}

impl SearchResult {
    pub fn value_to_string(&self) -> String {
        self.data_type.info().from_bytes(&self.value)
    }
}

/// Scans the process for matches.
pub fn scan(
    tx: Sender<SearchResult>,
    process: &Arc<Process>,
    value: &[u8],
    data_type: DataTypeEnum,
    to_filter: &Vec<SearchResult>,
) {
    if to_filter.is_empty() {
        scan_all(tx, process, value, data_type)
    } else {
        filter_addresses(tx, process, value, data_type, to_filter)
    }
}

/// Scans the entire process memory for matches.
fn scan_all(
    tx: Sender<SearchResult>,
    process: &Arc<Process>,
    value: &[u8],
    data_type: DataTypeEnum,
) {
    let vmem = process.query_vmem();
    for vpage in vmem {
        if vpage.size > MAX_PAGE_BYTES {
            panic!("Partial page scan not supported yet.")
        }
        let mem = process
            .get_mem_at(vpage.start, vpage.size)
            .unwrap_or_else(|_| vec![]);
        scan_page(&tx, &mem, vpage.start, value, data_type);
    }
}

/// Scans a single page in memory for matches.
fn scan_page(
    tx: &Sender<SearchResult>,
    mem: &Vec<u8>,
    page_start: usize,
    value: &[u8],
    data_type: DataTypeEnum,
) {
    let len = value.len();
    for (i, v) in mem.chunks_exact(len).enumerate() {
        if v == value {
            let result = SearchResult {
                address: i * len + page_start,
                data_type,
                value: v.to_vec(),
            };
            tx.send(result).unwrap();
        }
    }
}

/// Filters and existing list of addresses down to only those that match.
fn filter_addresses(
    tx: Sender<SearchResult>,
    process: &Arc<Process>,
    value: &[u8],
    data_type: DataTypeEnum,
    to_filter: &Vec<SearchResult>,
) {
    let data_type_size = data_type.info().size_of();
    for result in to_filter {
        let address = result.address;
        let v = process.get_mem_at(address, data_type_size);
        if let Ok(v) = v {
            if v == value {
                let result = SearchResult {
                    address,
                    data_type,
                    value: v.to_vec(),
                };
                tx.send(result).unwrap();
            }
        }
    }
}
