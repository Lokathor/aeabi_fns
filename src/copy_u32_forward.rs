use crate::*;

/// Copies `count` bytes from `src` to `dest`, going upward in address value.
///
/// Copies are done in 4-byte chunks as much as possible. If the number of bytes
/// to copy is not a multiple of 4 then the last portion will be done using a
/// 2-byte and/or 1-byte copy.
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
pub unsafe extern "C" fn copy_u32_forward(
  mut dest: *mut mu_u32, mut src: *const mu_u32, mut count: usize,
) {
  if count > 0 {
    debug_assert!(dest as usize % 4 == 0, "dest must be aligned to 4!");
    debug_assert!(src as usize % 4 == 0, "src must be aligned to 4!");
  }
  cfg_armv4t! {
    yes: {
      core::arch::asm! {
        // The loop reasoning here is similar to `copy_u8_forward`
        "1:",
        "subs    {count}, {count}, #4",
        "ldrge   {temp}, [{src}], #4",
        "strge   {temp}, [{dest}], #4",
        "bgt     1b",

        // temp = count << 31;
        // this puts bit 1 as the carry flag,
        // and bit 0 as the neg flag
        "lsls    {temp}, {count}, #31",
        // if count bit 1 set, copy 2
        "ldrhcs  {temp}, [{src}], #2",
        "strhcs  {temp}, [{dest}], #2",
        // if count bit 0 set, copy 1
        "ldrbmi  {temp}, [{src}], #1",
        "strbmi  {temp}, [{dest}], #1",

        dest = inout(reg) dest,
        src = inout(reg) src,
        count = inout(reg) count,
        temp = out(reg) _,
        options(nostack)
      }
    }
    no: {
      while count >= 4 {
        *dest = *src;
        dest = dest.add(1);
        src = src.add(1);
        count -= 4;
      }
      if (count & 0b10) != 0 {
        *dest.cast::<mu_u16>() = *src.cast::<mu_u16>();
        dest = dest.byte_add(2);
        src = src.byte_add(2);
      }
      if (count & 1) != 0 {
        *dest.cast::<mu_u8>() = *src.cast::<mu_u8>();
      }
    }
  }
}
