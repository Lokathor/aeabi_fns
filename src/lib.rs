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

// Note(Lokathor): Each individual function is a separate file for ease of
// tabbed viewing, because they're not very visually distinct when scrolling up
// and down within a single file.

mod copy_u8_forward;
pub use copy_u8_forward::copy_u8_forward;

mod copy_u8_backward;
pub use copy_u8_backward::copy_u8_backward;

mod copy_u16_forward;
pub use copy_u16_forward::copy_u16_forward;

mod copy_u16_backward;
pub use copy_u16_backward::copy_u16_backward;
