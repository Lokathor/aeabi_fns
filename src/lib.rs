#![no_std]
#![allow(unused_mut)]
#![allow(non_camel_case_types)]
#![allow(unsafe_op_in_unsafe_fn)]

use core::mem::MaybeUninit;

pub type mu_u8 = MaybeUninit<u8>;
pub type mu_u16 = MaybeUninit<u16>;
pub type mu_u32 = MaybeUninit<u32>;

macro_rules! cfg_arm7tdmi {
  ( yes: { $($yes_tokens:tt)* } no: { $($no_tokens:tt)* } ) => {
    #[cfg(all(target_arch="arm", feature="arm7tdmi"))]{
      $($yes_tokens)*
    }
    #[cfg(not(all(target_arch="arm", feature="arm7tdmi")))]{
      $($no_tokens)*
    }
  }
}

/// Copies `count` bytes from `src` to `dest`, going upward in address value.
///
/// ## Safety
/// * If `count` is zero, the `src` and `dest` pointers are not accessed, and
///   they can even be invalid or null.
/// * If `count` is non-zero, then both `src` and `dest` must be valid for
///   `count` bytes forward and one of the following must be true:
///   * The `src` and `dest` regions are entirely disjoint.
///   * `src` equals `dest` (there is exact overlap).
///   * `src` is *greater* than `dest` (a partial overlap).
#[inline]
#[cfg_attr(feature = "link_iwram", link_section = ".iwram.copy_u8_forward")]
pub unsafe extern "C" fn copy_u8_forward(
  mut dest: *mut mu_u8, mut src: *const mu_u8, mut count: usize,
) {
  cfg_arm7tdmi! {
    yes: {
      core::arch::asm! {
        "1:",
        "subs    {count}, {count}, #1",
        "ldrbge  {temp}, [{src}], #1",
        "strbge  {temp}, [{dest}], #1",
        "bgt     1b",
        dest = inout(reg) dest => _,
        src = inout(reg) src => _,
        count = inout(reg) count => _,
        temp = out(reg) _,
        options(nostack)
      }
    }
    no: {
      while count > 0 {
        *dest = *src;
        dest = dest.add(1);
        src = src.add(1);
        count -= 1;
      }
    }
  }
}
