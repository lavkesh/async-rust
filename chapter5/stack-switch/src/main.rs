use std::arch::asm;

const SSIZE: usize = 1024 * 64;

#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    sp: u64, // offset 0
    lr: u64, // offset 8 (x30)
}

fn hello() -> ! {
    println!("Hello, world!");
    loop {}
}

unsafe fn gt_switch(new: *const ThreadContext) -> ! {
    unsafe {
        asm!(
        "ldr x30, [x0, #8]", // lr = new.lr  (where `ret` will jump)
        "ldr x0,  [x0, #0]", // x0 = new.sp
        "mov sp, x0",        // sp can't be loaded directly
        "ret",               // jump to x30
        in("x0") new,
        options(noreturn)
        )
    }
}

fn main() {
    let mut ctx = ThreadContext::default();
    let mut stack = vec![0u8; SSIZE];

    unsafe {
        let stack_top = stack.as_mut_ptr().add(SSIZE);
        let sp_aligned = (stack_top as usize & !15) as *mut u64;

        let slot = sp_aligned.offset(-2);
        std::ptr::write(slot, hello as *const () as u64);

        ctx.sp = sp_aligned as u64;
        ctx.lr = hello as *const () as u64;

        println!("hello fn addr: {:#x}", hello as *const () as usize);
        println!("ctx.sp: {:#x}, ctx.lr: {:#x}", ctx.sp, ctx.lr);

        for i in 1..=6isize {
            let p = sp_aligned.offset(-i);
            println!("mem: {:#x}, val: {:#018x}", p as usize, *p);
        }

        gt_switch(&ctx);
    }
}