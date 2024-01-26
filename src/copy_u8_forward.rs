use crate::*;

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
/// * `count` may not exceed `isize::MAX as usize`. (All Rust allocations
///   already follow this rule, but perhaps it's worth stating that it is an
///   assumption of the function.)
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
