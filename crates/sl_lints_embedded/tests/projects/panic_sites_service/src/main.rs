//! A firmware-like binary with one panic site of each kind, built for thumbv7em with debug info. Its ELF,
//! `panic_sites.elf`, is checked in; `just panic-fixture` rebuilds it with the pinned toolchain.
#![no_std]
#![no_main]

mod led;
mod power;

/// A value the compiler cannot see through
fn input(n: usize) -> usize {
    // SAFETY: a stack local read through a volatile pointer, only to hide the value from the optimizer
    unsafe { core::ptr::read_volatile(&raw const n) }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let mut acc = 0u32;
    loop {
        acc = acc.wrapping_add(u32::from(led::brightness(input(3))));
        acc = acc.wrapping_add(power::duty(input(7) as u32, input(2) as u32));
        acc = acc.wrapping_add(power::scaled::<u8>(input(1) as u8));
        acc = acc.wrapping_add(power::scaled::<u16>(input(1) as u16));
        power::arm(input(1) == 1);
        acc = acc.wrapping_add(power::window(&[1, 2, 3, 4], input(1)));
        core::hint::black_box(acc);
    }
}

/// Where the last panic was, as a crash record would keep it
static mut LAST: (usize, u32) = (0, 0);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    let at = info.location().map_or((0, 0), |l| (l.file().as_ptr() as usize, l.line()));
    // SAFETY: written only here, and the handler never returns
    unsafe { core::ptr::write_volatile(&raw mut LAST, at) };
    loop {
        core::hint::black_box(());
    }
}
