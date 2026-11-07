//! This process's own CPU time and working set.
//!
//! Measuring a watcher's idle cost needs the process's CPU time, and the task
//! allows no new dependency for it, so the three Win32 calls are declared here
//! instead of pulling in `windows-sys`. On any other platform both functions
//! return `None` and the tables print `n/a`.

use std::time::Duration;

#[cfg(windows)]
pub fn cpu_time() -> Option<Duration> {
    let mut creation = FileTime::default();
    let mut exit = FileTime::default();
    let mut kernel = FileTime::default();
    let mut user = FileTime::default();
    // SAFETY: GetCurrentProcess returns a pseudo handle valid for the calling
    // process, and every pointer is a live local for the duration of the call.
    let ok = unsafe {
        GetProcessTimes(
            GetCurrentProcess(),
            &mut creation,
            &mut exit,
            &mut kernel,
            &mut user,
        )
    };
    if ok == 0 {
        return None;
    }
    // FILETIME counts 100 ns ticks; a process cannot use enough of them to
    // overflow a u64 in any realistic run, and saturating keeps it safe anyway.
    let ticks = kernel.ticks().saturating_add(user.ticks());
    Some(Duration::from_nanos(ticks.saturating_mul(100)))
}

#[cfg(windows)]
pub fn working_set_bytes() -> Option<u64> {
    let mut counters = MemoryCounters::default();
    // SAFETY: the counters struct is a live local, its `cb` is set to its own
    // size as the API requires, and the handle is this process's.
    let ok = unsafe {
        GetProcessMemoryInfo(
            GetCurrentProcess(),
            &mut counters,
            std::mem::size_of::<MemoryCounters>() as u32,
        )
    };
    (ok != 0).then_some(counters.working_set_size as u64)
}

#[cfg(windows)]
#[repr(C)]
#[derive(Default)]
struct FileTime {
    low: u32,
    high: u32,
}

#[cfg(windows)]
impl FileTime {
    fn ticks(&self) -> u64 {
        (u64::from(self.high) << 32) | u64::from(self.low)
    }
}

/// `PROCESS_MEMORY_COUNTERS` from psapi.h: DWORD, DWORD, then SIZE_T fields.
#[cfg(windows)]
#[repr(C)]
#[derive(Default)]
struct MemoryCounters {
    cb: u32,
    page_fault_count: u32,
    peak_working_set_size: usize,
    working_set_size: usize,
    quota_peak_paged_pool_usage: usize,
    quota_paged_pool_usage: usize,
    quota_peak_non_paged_pool_usage: usize,
    quota_non_paged_pool_usage: usize,
    pagefile_usage: usize,
    peak_pagefile_usage: usize,
}

#[cfg(windows)]
type Handle = *mut std::ffi::c_void;

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> Handle;
    fn GetProcessTimes(
        process: Handle,
        creation: *mut FileTime,
        exit: *mut FileTime,
        kernel: *mut FileTime,
        user: *mut FileTime,
    ) -> i32;
}

#[cfg(windows)]
#[link(name = "psapi")]
extern "system" {
    fn GetProcessMemoryInfo(process: Handle, counters: *mut MemoryCounters, cb: u32) -> i32;
}

#[cfg(not(windows))]
pub fn cpu_time() -> Option<Duration> {
    None
}

#[cfg(not(windows))]
pub fn working_set_bytes() -> Option<u64> {
    None
}
