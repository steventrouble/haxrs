use super::Process;

const MAX_SCAN_BYTES: usize = 0x20000000; // 512 MiB

/// Scans the entire process for a match.
pub fn scan(process: &Process, value: &[u8]) -> Vec<usize> {
    let mut addresses = vec![];
    let vmem = process.query_vmem();
    for vpage in vmem {
        if vpage.size > MAX_SCAN_BYTES {
            panic!("Partial page scan not supported yet.")
        }
        let mem = process
            .get_mem_at(vpage.start, vpage.size)
            .unwrap_or_else(|_| vec![]);
        addresses.append(&mut scan_page(&mem, vpage.start, value));
    }
    addresses
}

fn scan_page(mem: &Vec<u8>, page_start: usize, value: &[u8]) -> Vec<usize> {
    let len = value.len();
    let mut addresses = Vec::with_capacity(1024);
    for (i, v) in mem.chunks_exact(len).enumerate() {
        if v == value {
            addresses.push(i*len+page_start);
        }
    }
    addresses
}
