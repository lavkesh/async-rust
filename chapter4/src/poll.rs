use crate::ffi::{
    EV_ADD, EV_CLEAR, EV_DELETE, EV_ENABLE, EV_FILTER_READ, EV_FILTER_WRITE, Timespec, close,
    kevent, kqueue,
};
use std::os::fd::AsRawFd;
use std::os::raw::{c_int, c_long, c_short, c_void};
use std::time::Duration;
use std::{io, ptr};

pub type Events = Vec<kevent>;

pub struct Poll {
    registry: Registry,
}

impl Poll {
    pub fn new() -> io::Result<Self> {
        let res = unsafe { kqueue() };
        if res < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            registry: Registry { raw_fd: res },
        })
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn poll(&self, events: &mut Events, timeout: Option<Duration>) -> io::Result<()> {
        if events.capacity() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "events capacity must be greater than 0",
            ));
        }

        events.clear();

        let fd = self.registry.raw_fd;
        let time_spec = timeout.map(|t| Timespec {
            tv_sec: t.as_secs() as c_long,
            tv_nsec: t.subsec_nanos() as c_long,
        });
        let time_spec_ptr: *const Timespec = match &time_spec {
            Some(ts) => ts as *const Timespec,
            None => ptr::null(),
        };

        let res = unsafe {
            kevent(
                fd,
                ptr::null(),
                0,
                events.as_mut_ptr(),
                events.capacity() as c_int,
                time_spec_ptr,
            )
        };

        if res < 0 {
            let err = io::Error::last_os_error();
            if err.kind() == io::ErrorKind::Interrupted {
                return Ok(());
            }
            return Err(err);
        }
        unsafe { events.set_len(res as usize) };
        Ok(())
    }
}

pub struct Registry {
    raw_fd: c_int,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Interest: u8 {
        const READABLE = 0b0001;
        const WRITABLE = 0b0010;
    }
}

impl Registry {
    fn get_change(fd: c_int, token: usize, filter: c_short, flags: u16) -> kevent {
        kevent {
            ident: fd as usize,
            filter,
            flags,
            fflags: 0,
            data: 0,
            udata: token as *mut c_void,
        }
    }

    fn submit(&self, changes: &[kevent]) -> io::Result<()> {
        let res = unsafe {
            kevent(
                self.raw_fd,
                changes.as_ptr(),
                changes.len() as c_int,
                ptr::null_mut(),
                0,
                ptr::null(),
            )
        };
        if res < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    fn build_changes(fd: c_int, token: usize, interests: Interest, flags: u16) -> Vec<kevent> {
        let mut changes = Vec::with_capacity(2);
        if interests.contains(Interest::READABLE) {
            changes.push(Self::get_change(fd, token, EV_FILTER_READ, flags));
        }
        if interests.contains(Interest::WRITABLE) {
            changes.push(Self::get_change(fd, token, EV_FILTER_WRITE, flags));
        }
        changes
    }

    pub fn register(
        &self,
        source: &impl AsRawFd,
        token: usize,
        interests: Interest,
    ) -> io::Result<()> {
        let flags = EV_ADD | EV_ENABLE | EV_CLEAR;
        let changes = Self::build_changes(source.as_raw_fd(), token, interests, flags);
        if changes.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "at least one interest is required",
            ));
        }
        self.submit(&changes)
    }

    pub fn deregister(&self, source: &impl AsRawFd, interests: Interest) -> io::Result<()> {
        let changes = Self::build_changes(source.as_raw_fd(), 0, interests, EV_DELETE);
        if changes.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "at least one interest is required",
            ));
        }
        self.submit(&changes)
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        let res = unsafe { close(self.raw_fd) };
        if res < 0 {
            let err = io::Error::last_os_error();
            eprintln!("ERROR closing kqueue fd: {err:?}");
        }
    }
}
