use std::os::raw::{c_int, c_short, c_uint, c_long, c_void};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kevent {
    pub ident: usize,       // uintptr_t: 8 bytes
    pub filter: c_short,    // int16_t:   2 bytes
    pub flags: u16,         // uint16_t:  2 bytes
    pub fflags: c_uint,     // uint32_t:  4 bytes
    pub data: isize,        // intptr_t:  8 bytes
    pub udata: *mut c_void, // void*:     8 bytes
}

#[repr(C)]
pub struct Timespec {
    pub tv_sec: c_long,
    pub tv_nsec: c_long,
}

pub const EV_FILTER_READ: i16 = -1;
pub const EV_FILTER_WRITE: i16 = -2;

pub const EV_ADD: u16 = 0x0001;
pub const EV_DELETE: u16 = 0x0002;
pub const EV_ENABLE: u16 = 0x0004;
//pub const EV_DISABLE: u16 = 0x0008;
pub const EV_CLEAR: u16 = 0x0020;
//pub const EV_ONESHOT: u16 = 0x0010;

pub const EV_EOF: u16 = 0x8000;
pub const EV_ERROR: u16 = 0x4000;

#[link(name = "c")]
unsafe extern "C" {
    pub fn kqueue() -> c_int;

    pub fn kevent(
        kq: c_int,
        changelist: *const kevent,
        nchanges: c_int,
        eventlist: *mut kevent,
        nevents: c_int,
        timeout: *const Timespec,
    ) -> c_int;

    pub fn close(fd: c_int) -> c_int;
}