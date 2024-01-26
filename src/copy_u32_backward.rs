use crate::*;

/// Copies `count` bytes from `src` to `dest`, starting at one-past-the-end.
///
/// Copies are done in 4-byte chunks as much as possible. If the number of bytes
/// to copy is not a multiple of 4 then the last portion will be done using a
/// 2-byte and/or 1-byte copy.
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
/// * `count` may not exceed `isize::MAX as usize`. (All Rust allocations
///   already follow this rule, but perhaps it's worth stating that it is an
///   assumption of the function.)
#[inline]
#[cfg_attr(feature = "link_iwram", link_section = ".iwram.copy_u8_backward")]
#[cfg_attr(
  all(target_arch = "arm", target_feature = "thumb-mode", feature = "armv4t"),
  instruction_set(arm::a32)
)]
pub unsafe extern "C" fn copy_u32_backward(
  mut dest: *mut mu_u32, mut src: *const mu_u32, mut count: usize,
) {
  if count > 0 {
    debug_assert!(dest as usize % 4 == 0, "dest must be aligned to 4!");
    debug_assert!(src as usize % 4 == 0, "src must be aligned to 4!");
  }
  // IMPORTANT: in the backward loop we adjust the pointers *before* the copy,
  // instead of after the copy like the forward loop does.
  cfg_armv4t! {
    yes: {
      // The loop reasoning here is similar to `copy_u8_backward`
      core::arch::asm! {
        "1:",
        "subs    {count}, {count}, #4",
        "ldrhge  {temp}, [{src}, #-4]!",
        "strhge  {temp}, [{dest}, #-4]!",
        "bgt     1b",

        // temp = count << 31;
        // this puts bit 1 as the carry flag,
        // and bit 0 as the neg flag
        "lsls    {temp}, {count}, #31",
        // if count bit 1 set, copy 2
        "ldrhge  {temp}, [{src}, #-2]!",
        "strhge  {temp}, [{dest}, #-2]!",
        // if count bit 0 set, copy 1
        "ldrbge  {temp}, [{src}, #-1]!",
        "strbge  {temp}, [{dest}, #-1]!",

        dest = inout(reg) dest,
        src = inout(reg) src,
        count = inout(reg) count,
        temp = out(reg) _,
        options(nostack)
      }
    }
    no: {
      while count >= 4 {
        dest = dest.sub(1);
        src = src.sub(1);
        *dest = *src;
        count -= 4;
      }
      if (count & 0b10) != 0 {
        dest = dest.byte_sub(2);
        src = src.byte_sub(2);
        *dest.cast::<mu_u16>() = *src.cast::<mu_u16>();
      }
      if (count & 1) != 0 {
        dest = dest.byte_sub(1);
        src = src.byte_sub(1);
        *dest.cast::<mu_u8>() = *src.cast::<mu_u8>();
      }
    }
  }
}
