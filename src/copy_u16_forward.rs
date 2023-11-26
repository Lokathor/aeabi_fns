use crate::*;

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
