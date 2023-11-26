use crate::*;

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
  if (count & 1) != 0 {
    let mut dest = dest.cast::<mu_u8>();
    let mut src = src.cast::<mu_u8>();
    dest = dest.sub(1);
    src = src.sub(1);
    *dest = *src;
  }
}
