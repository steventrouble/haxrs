use std::mem;
use windows::Win32::Foundation as win;
use windows::Win32::System as winsys;
use windows::Win32::System::Diagnostics::Debug as windbg;
use windows::Win32::System::Memory as winmem;
use windows::Win32::System::Threading::PROCESS_VM_WRITE;
use winsys::ProcessStatus;
use winsys::Threading::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};

/// Represents a single process object running on the system.
pub struct Process {
    handle: win::HANDLE,
}

impl Process {
    /// Open a new process using the windows API.
    pub fn new(pid: u32) -> Result<Process, Box<dyn std::error::Error>> {
        let handle = unsafe {
            winsys::Threading::OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_VM_READ | PROCESS_VM_WRITE,
                win::BOOL::from(true),
                pid,
            )
        }?;
        Ok(Process { handle })
    }

    /// Gets a list of all the running processes.
    pub fn get_processes() -> Result<Vec<Process>, String> {
        let mut needed_bytes: u32 = 1024;
        let mut used_bytes: u32 = 0;

        for _ in 0..2 {
            let num_entries: usize = needed_bytes as usize / mem::size_of::<u32>();
            let mut pids: Vec<u32> = vec![0; num_entries];
            let success: bool;

            unsafe {
                success = ProcessStatus::K32EnumProcesses(
                    pids.as_mut_ptr(),
                    needed_bytes,
                    &mut used_bytes,
                )
                .as_bool();
            }

            // try again if the vec was too small
            if needed_bytes == used_bytes {
                needed_bytes = used_bytes + 128;
                continue;
            }

            if !success {
                break;
            }

            pids.truncate((used_bytes as usize) / std::mem::size_of::<u32>());
            return Ok(to_processes(pids));
        }

        let error = unsafe { win::GetLastError() }.0;
        return Err(format!("Got error {error}"));
    }

    /// Returns the name (e.g. "example.exe") of the process.
    pub fn get_name(&self) -> String {
        const SIZE: usize = 128;
        let mut bytes: [u16; SIZE] = [0; SIZE];
        unsafe {
            winsys::ProcessStatus::K32GetProcessImageFileNameW(self.handle, &mut bytes);
        }
        let path = String::from_utf16(&bytes).unwrap_or_else(|_| "UNKNOWN".to_string());
        path.split("\\").last().unwrap().to_string()
    }

    /// Returns the value of the memory at the given address.
    /// Uses strings because currently only end-users will interact with this.
    pub fn get_mem_at(&self, addr: usize, num_bytes: usize) -> Result<Vec<u8>, String> {
        let mut val : Vec<u8> = vec![0; num_bytes];
        let mut bytes_read: usize = 0;
        let success = unsafe {
            windbg::ReadProcessMemory(
                self.handle,
                addr as _,
                val.as_mut_ptr() as _,
                num_bytes,
                &mut bytes_read,
            )
        }
        .as_bool();
        if !success || bytes_read != num_bytes {
            return Err("Could not read address".to_string());
        }
        Ok(val)
    }

    /// Sets the value of the memory at the given address.
    /// Uses strings because currently only end-users will interact with this.
    pub fn set_mem_at(&self, addr: usize, bytes: Vec<u8>) -> Result<(), String> {
        let mut bytes_written: usize = 0;
        let success = unsafe {
            windbg::WriteProcessMemory(
                self.handle,
                addr as _,
                bytes.as_ptr() as _,
                bytes.len(),
                &mut bytes_written,
            )
        }
        .as_bool();
        if !success || bytes_written != bytes.len() {
            return Err("Could not write address".to_string());
        }
        Ok(())
    }

    /// Get the list of all the process's pages in virtual memory.
    pub fn query_vmem(&self) -> Vec<VirtualPage> {
        let mut pages: Vec<VirtualPage> = vec![];
        let mut current: usize = 0;
        for _ in 0..20000 {
            let mut a = winmem::MEMORY_BASIC_INFORMATION::default();
            let res = unsafe {
                winmem::VirtualQueryEx(
                    self.handle,
                    current as _,
                    &mut a,
                    std::mem::size_of::<winmem::MEMORY_BASIC_INFORMATION>(),
                )
            };

            if res == 0 {
                break;
            }

            let base = a.BaseAddress as usize;
            let region_size = a.RegionSize;
            let next = base + region_size;
            let committed = (a.State.0 & winmem::MEM_COMMIT.0) != 0x0;
            if committed && writable(a.Protect.0) {
                pages.push(VirtualPage {
                    start: base,
                    size: region_size,
                });
            }

            if next < current || base > 0x7FFFFFFFFFFF {
                break;
            }
            current = next;
        }
        pages
    }
}

/// Returns true if the memory region is writable.
fn writable(protect_flags: u32) -> bool {
    (protect_flags & (winmem::PAGE_READWRITE.0 | winmem::PAGE_EXECUTE_READWRITE.0)) != 0
}

/// We *must* remember to close the handle when we're done.
impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            win::CloseHandle(self.handle);
        }
    }
}

/// Converts a list of PIDs into processes.
fn to_processes(pids: Vec<u32>) -> Vec<Process> {
    pids.iter()
        .filter_map(|&pid| Process::new(pid).ok())
        .collect()
}

/// Info about a memory page.
pub struct VirtualPage {
    pub start: usize,
    pub size: usize,
}
