//#![no_std]
#![allow(unused_mut)]
#![allow(non_camel_case_types)]
#![allow(unsafe_op_in_unsafe_fn)]

use core::mem::MaybeUninit;

pub type mu_u8 = MaybeUninit<u8>;
pub type mu_u16 = MaybeUninit<u16>;
pub type mu_u32 = MaybeUninit<u32>;

/// This macro makes it easy to pick between `armv4t`-specific inline asm and
/// the Rust-only fallback version.
macro_rules! cfg_armv4t {
  (
    // this is approximately how I expect you'd format the macro invocation
    yes: {
      $($yes_tokens:tt)*
    }
    no: {
      $($no_tokens:tt)*
    }
  ) => {
    #[cfg(all(target_arch="arm", feature="armv4t"))]
    {
      $($yes_tokens)*
    }
    #[cfg(not(all(target_arch="arm", feature="armv4t")))]
    {
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
#[cfg_attr(
  all(target_arch = "arm", target_feature = "thumb-mode", feature = "armv4t"),
  instruction_set(arm::a32)
)]
pub unsafe extern "C" fn copy_u8_forward(
  mut dest: *mut mu_u8, mut src: *const mu_u8, mut count: usize,
) {
  cfg_armv4t! {
    yes: {
      // This loop assumes that the count is non-zero to start, and so it always
      // updates `count`, followed by a conditional copy and continue.
      // * Pro: 8 bytes less code in the binary
      // * Pro: save 2 cycles on non-zero sized copies
      // * Con: lose 3 cycles on zero sized copies.
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
      while count >= 1 {
        *dest = *src;
        dest = dest.add(1);
        src = src.add(1);
        count -= 1;
      }
    }
  }
}

/// Copies `count` bytes from `src` to `dest`, going upward in address value.
///
/// Copies are done in 2-byte chunks as much as possible. If the number of bytes
/// to copy is odd then the last copy will be a single-byte copy.
///
/// ## Safety
/// * If `count` is zero, the `src` and `dest` pointers are not accessed, and
///   they can even be invalid or null.
/// * If `count` is non-zero, then both `src` and `dest` must be aligned, valid
///   for `count` bytes forward, and one of the following must be true:
///   * The `src` and `dest` regions are entirely disjoint.
///   * `src` equals `dest` (there is exact overlap).
///   * `src` is *greater* than `dest` (a partial overlap).
#[inline]
#[cfg_attr(feature = "link_iwram", link_section = ".iwram.copy_u8_forward")]
#[cfg_attr(
  all(target_arch = "arm", target_feature = "thumb-mode", feature = "armv4t"),
  instruction_set(arm::a32)
)]
pub unsafe extern "C" fn copy_u16_forward(
  mut dest: *mut mu_u16, mut src: *const mu_u16, mut count: usize,
) {
  eprintln!("=== start count: {count}");
  cfg_armv4t! {
    yes: {
      // The loop reasoning here is similar to `copy_u8_forward`
      core::arch::asm! {
        "1:",
        "subs    {count}, {count}, #2",
        "ldrhge  {temp}, [{src}], #2",
        "strhge  {temp}, [{dest}], #2",
        "bgt     1b",
        dest = inout(reg) dest,
        src = inout(reg) src,
        count = inout(reg) count,
        temp = out(reg) _,
        options(nostack)
      }
    }
    no: {
      while count >= 2 {
        *dest = *src;
        dest = dest.add(1);
        src = src.add(1);
        count -= 2;
      }
    }
  }
  eprintln!("post-loop count: {count}");
  if (count & 1) != 0 {
    eprintln!("copying 1 byte");
    let dest = dest.cast::<mu_u8>();
    let src = src.cast::<mu_u8>();
    *dest = *src;
    // skip adjusting count for the last byte, it's the end of the fn
  }
}

/// Copies `count` bytes from `src` to `dest`, starting at one-past-the-end.
///
/// ## Safety
/// * If `count` is zero, the `src` and `dest` pointers are not accessed, and
///   they can even be invalid or null.
/// * If `count` is non-zero, then both `src` and `dest` must be the
///   one-past-the-end pointers for `count` bytes backward and one of the
///   following must be true:
///   * The `src` and `dest` regions are entirely disjoint.
///   * `src` equals `dest` (there is exact overlap).
///   * `src` is *less* than `dest` (a partial overlap).
#[inline]
#[cfg_attr(feature = "link_iwram", link_section = ".iwram.copy_u8_backward")]
#[cfg_attr(
  all(target_arch = "arm", target_feature = "thumb-mode", feature = "armv4t"),
  instruction_set(arm::a32)
)]
pub unsafe extern "C" fn copy_u8_backward(
  mut dest: *mut mu_u8, mut src: *const mu_u8, mut count: usize,
) {
  // IMPORTANT: in the backward loop we adjust the pointers *before* the copy,
  // instead of after the copy like the forward loop does.
  cfg_armv4t! {
    yes: {
      // The loop reasoning here is similar to `copy_u8_forward`
      core::arch::asm! {
        "1:",
        "subs    {count}, {count}, #1",
        "ldrbge  {temp}, [{src}, #-1]!",
        "strbge  {temp}, [{dest}, #-1]!",
        "bgt     1b",
        dest = inout(reg) dest => _,
        src = inout(reg) src => _,
        count = inout(reg) count => _,
        temp = out(reg) _,
        options(nostack)
      }
    }
    no: {
      while count >= 1 {
        dest = dest.sub(1);
        src = src.sub(1);
        *dest = *src;
        count -= 1;
      }
    }
  }
}

/// Copies `count` bytes from `src` to `dest`, starting at one-past-the-end.
///
/// Copies are done in 2-byte chunks as much as possible. If the number of bytes
/// to copy is odd then the last copy will be a single-byte copy.
///
/// ## Safety
/// * If `count` is zero, the `src` and `dest` pointers are not accessed, and
///   they can even be invalid or null.
/// * If `count` is non-zero, then both `src` and `dest` must be aligned, be the
///   one-past-the-end pointers for `count` bytes backward, and one of the
///   following must be true:
///   * The `src` and `dest` regions are entirely disjoint.
///   * `src` equals `dest` (there is exact overlap).
///   * `src` is *less* than `dest` (a partial overlap).
#[inline]
#[cfg_attr(feature = "link_iwram", link_section = ".iwram.copy_u8_backward")]
#[cfg_attr(
  all(target_arch = "arm", target_feature = "thumb-mode", feature = "armv4t"),
  instruction_set(arm::a32)
)]
pub unsafe extern "C" fn copy_u16_backward(
  mut dest: *mut mu_u16, mut src: *const mu_u16, mut count: usize,
) {
  // IMPORTANT: in the backward loop we adjust the pointers *before* the copy,
  // instead of after the copy like the forward loop does.
  cfg_armv4t! {
    yes: {
      // The loop reasoning here is similar to `copy_u8_forward`
      core::arch::asm! {
        "1:",
        "subs    {count}, {count}, #2",
        "ldrhge  {temp}, [{src}, #-2]!",
        "strhge  {temp}, [{dest}, #-2]!",
        "bgt     1b",
        dest = inout(reg) dest,
        src = inout(reg) src,
        count = inout(reg) count,
        temp = out(reg) _,
        options(nostack)
      }
    }
    no: {
      while count >= 2 {
        dest = dest.sub(1);
        src = src.sub(1);
        *dest = *src;
        count -= 2;
      }
    }
  }
  if count != 0 {
    let mut dest = dest.cast::<mu_u8>();
    let mut src = src.cast::<mu_u8>();
    dest = dest.sub(1);
    src = src.sub(1);
    *dest = *src;
  }
}
