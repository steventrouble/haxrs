use std::mem;
use windows::Win32::Foundation as win;
use windows::Win32::System as winsys;
use windows::Win32::System::Diagnostics::Debug as windbg;
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
