//! Process management syscalls

use crate::{
    mm::{translated_pa, AccessType},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_syscall_count, mmap,
        munmap, suspend_current_and_run_next,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    if let Some(pa) = translated_pa::<TimeVal>(current_user_token(), _ts, AccessType::Write) {
        unsafe {
            let us = get_time_us();
            *pa = TimeVal {
                sec: us / 1_000_000,
                usec: us % 1_000_000,
            };
            0
        }
    } else {
        -1
    }
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    let token = current_user_token();
    match _trace_request {
        0 => {
            if let Some(pa) = translated_pa::<u8>(token, _id as *const u8, AccessType::Read) {
                unsafe { *pa as isize }
            } else {
                -1
            }
        }
        1 => {
            if let Some(pa) = translated_pa::<u8>(token, _id as *const u8, AccessType::Write) {
                unsafe { *pa = _data as u8 };
                0
            } else {
                -1
            }
        }
        2 => get_syscall_count(_id) as isize,
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _prot: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if _prot & !0x7 != 0 || _prot & 0x7 == 0  {
        return -1;
    }
    mmap(_start, _len, _prot)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if _len == 0 {
        return 0;
    }
    munmap(_start, _len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
