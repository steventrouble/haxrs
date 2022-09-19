use std::sync::{mpsc::Sender, Arc};

use crate::parser::Node;

use super::{Process, DataTypeEnum};

const MAX_PAGE_BYTES: usize = 1 << 29; // 512 MiB

#[derive(Clone)]
pub struct SearchResult {
    pub address: usize,
    pub data_type: DataTypeEnum,
    pub value: Vec<u8>,
}

impl SearchResult {
    pub fn value_to_string(&self) -> String {
        self.data_type.info().display(&self.value)
    }
}

/// Scans the process for matches.
pub fn scan(
    tx: Sender<SearchResult>,
    process: &Arc<Process>,
    query: Node,
    to_filter: &Vec<SearchResult>,
) {
    if to_filter.is_empty() {
        scan_all(tx, process, query)
    } else {
        filter_addresses(tx, process, query, to_filter)
    }
}

/// Scans the entire process memory for matches.
fn scan_all(tx: Sender<SearchResult>, process: &Arc<Process>, query: Node) {
    let vmem = process.query_vmem();
    for vpage in vmem {
        if vpage.size > MAX_PAGE_BYTES {
            panic!("Partial page scan not supported yet.")
        }
        let mem = process
            .get_mem_at(vpage.start, vpage.size)
            .unwrap_or_else(|_| vec![]);
        query.scan_page(&tx, &mem, vpage.start);
    }
}

/// Filters and existing list of addresses down to only those that match.
fn filter_addresses(
    tx: Sender<SearchResult>,
    process: &Arc<Process>,
    query: Node,
    previous_results: &Vec<SearchResult>,
) {
    for result in previous_results {
        let v = process.get_mem_at(result.address, result.data_type.info().size_of());
        if let Ok(v) = v {
            if query.test_single(&v, result.data_type) {
                let result = SearchResult {
                    value: v,
                    ..*result
                };
                tx.send(result).unwrap();
            }
        }
    }
}
