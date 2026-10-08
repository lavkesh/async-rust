use std::arch::asm;
use std::io;
use std::io::Error;
/*
#[inline(never)]
fn syscall(message: String) {
    let msg_ptr = message.as_ptr();
    let len = message.len();

    unsafe {
        asm!(
        "mov rax, 1",
        "mov rdi, 1",
        "syscall",
        in("rsi") msg_ptr,
        in("rdx") len,
        out("rax") _,
        out("rdi") _,
        lateout("rsi") _,
        lateout("rdx") _
        );
    }
}

 */
#[inline(never)]
fn syscall(message: String) {
    let ptr = message.as_ptr();
    let len = message.len();
    unsafe {
        asm!(
            "mov x16, 4", // write syscall is 4
            "mov x0, 1", // stdout, argument register
            "svc 0", // software interrupt
            in("x1") ptr, // write address to the buffer where the message is stored
            in("x2") len, // length
            out("x16") _,
            out("x0") _,
            lateout("x1") _,
            lateout("x2") _,
        )
    }
}
#[cfg(target_family = "unix")]
#[link(name = "c")]
unsafe extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> i32;
}
fn normal_syscall(message: String) -> io::Result<()> {
    let ptr = message.as_ptr();
    let len = message.len();
    let res = unsafe { write(1, ptr, len) };
    if res == -1 {
        return Err(Error::last_os_error());
    }
    Ok(())
}
fn main() {
    let message = "Hello world!\n";
    syscall(String::from(message));
    normal_syscall(String::from(message)).unwrap()
}
