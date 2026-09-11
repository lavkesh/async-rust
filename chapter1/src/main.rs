use std::arch::asm;

fn main() {
    let t = 100;
    let t_ptr: *const usize = &t;
    let res: usize = deref(t_ptr);
    println!("Hello, world! {}", res);
}
#[cfg(target_arch = "aarch64")]
fn deref(t_ptr: *const usize) -> usize {
    let mut res: usize = 0;
    unsafe {
        asm!("ldr {0}, [{1}]", out(reg) res, in(reg) t_ptr);
    }
    res
}
#[cfg(target_arch = "x86_64")]
fn deref(t_ptr: *const usize) -> usize {
    let mut res: usize = 0;
    unsafe {
        asm!("mov {0}, [{1}]", out(reg) res, in(reg) t_ptr);
    }
    res
}
