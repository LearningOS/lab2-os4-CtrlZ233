//! Process management syscalls

use crate::config::{MAX_SYSCALL_NUM, PAGE_SIZE_BITS};
use crate::mm::translated_refmut;
use crate::task::{current_user_token, exit_current_and_run_next, suspend_current_and_run_next,
                  TaskStatus, get_current_start_time, get_current_syscall_time, map_current_memory_set,
                  unmap_current_memory_set
};
use crate::task::TaskStatus::Running;
use crate::timer::{get_time_ms, get_time_us};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[derive(Clone, Copy)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

// YOUR JOB: 引入虚地址后重写 sys_get_time
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let _us = get_time_us();
    // println!("[kernel] _us: {}", _us);
    let token = current_user_token();
    *translated_refmut(token, _ts) = TimeVal {
        sec: _us / 1_000_000,
        usec: _us % 1_000_000,
    };
    0
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    // 页对齐
    if _start & ((1 << (PAGE_SIZE_BITS)) - 1) != 0 {
        return -1;
    }
    if (_port & !0x7 != 0) || (_port & 0x7 == 0) {
        return -1;
    }
    if map_current_memory_set(_start, _len, _port) {
        return 0;
    }
    -1
}

pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    // 页对齐
    if _start & ((1 << (PAGE_SIZE_BITS)) - 1) != 0 {
        return -1;
    }
    if unmap_current_memory_set(_start, _len) {
        return 0;
    }
    -1
}

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let token = current_user_token();
    *translated_refmut(token, ti) = TaskInfo {
        status: Running,
        syscall_times: get_current_syscall_time(),
        time: get_time_ms() - get_current_start_time(),
    };
    0
}
