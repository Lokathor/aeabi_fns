use aeabi_fns::{
  copy_u16_backward, copy_u16_forward, copy_u32_backward, copy_u32_forward,
  copy_u8_backward, copy_u8_forward,
};

// Note(Lokathor): Different base types on the vecs to get different minimum
// alignments on the buffer.

#[allow(dead_code)]
fn rand_bytes(n: usize) -> Vec<u8> {
  let mut v = vec![0_u8; n];
  getrandom::getrandom(&mut v).unwrap();
  v
}

#[allow(dead_code)]
fn rand_halfwords(n: usize) -> Vec<u16> {
  let mut v = vec![0_u16; n];
  getrandom::getrandom(bytemuck::cast_slice_mut(&mut v)).unwrap();
  v
}

#[allow(dead_code)]
fn rand_words(n: usize) -> Vec<u32> {
  let mut v = vec![0_u32; n];
  getrandom::getrandom(bytemuck::cast_slice_mut(&mut v)).unwrap();
  v
}

fn rand_u32() -> u32 {
  let mut bytes = [0; 4];
  getrandom::getrandom(&mut bytes).unwrap();
  u32::from_ne_bytes(bytes)
}

struct Lcg(u32);
impl Lcg {
  fn new() -> Self {
    Self(rand_u32())
  }
  fn next_u32(&mut self) -> u32 {
    self.0 = self.0.wrapping_mul(747796405).wrapping_add(1);
    self.0
  }
}

#[test]
fn test_copy_u8_forward() {
  let mut lcg = Lcg::new();

  // disjoint regions is the basic use case
  for len in 0..=16_usize {
    let src = rand_bytes(64);
    for s in 0..len {
      for d in 0..len {
        let mut dest_expected = vec![0; 64];
        let mut dest_actual = vec![0; 64];
        let b = (lcg.next_u32() % 16) as usize;
        let d_start = b + d;
        let d_end = d_start + len;
        let s_start = b + s;
        let s_end = s_start + len;
        dest_expected[d_start..d_end].copy_from_slice(&src[s_start..s_end]);
        unsafe {
          copy_u8_forward(
            dest_actual.as_mut_ptr().add(d_start).cast(),
            src.as_ptr().add(s_start).cast(),
            len,
          )
        }
        assert_eq!(dest_expected, dest_actual);
      }
    }
  }

  // src == dest is allowed for simplicity (but has no effect)
  for len in 0..=16_usize {
    let base = rand_bytes(64);
    for s in 0..len {
      let mut new = base.clone();
      unsafe {
        let p = new.as_mut_ptr();
        copy_u8_forward(p.add(s).cast(), p.add(s).cast(), len)
      }
      assert_eq!(base, new);
    }
  }

  // src > dest works even when the regions overlap
  for len in 0..=16_usize {
    let src = rand_bytes(64);
    for s in 0..len {
      for d in 0..len {
        if s < d {
          continue;
        }
        let mut out_expected = src.clone();
        let mut out_actual = src.clone();
        let b = (lcg.next_u32() % 16) as usize;
        let d_start = b + d;
        //let d_end = d_start + len;
        let s_start = b + s;
        let s_end = s_start + len;
        out_expected.copy_within(s_start..s_end, d_start);
        unsafe {
          let p = out_actual.as_mut_ptr();
          copy_u8_forward(p.add(d_start).cast(), p.add(s_start).cast(), len)
        }
        assert_eq!(out_expected, out_actual);
      }
    }
  }
}

#[test]
fn test_copy_u8_backward() {
  let mut lcg = Lcg::new();

  // disjoint regions is the basic use case
  for len in 0..=16_usize {
    let src = rand_bytes(64);
    for s in 0..len {
      for d in 0..len {
        let mut dest_expected = vec![0; 64];
        let mut dest_actual = vec![0; 64];
        let b = (lcg.next_u32() % 16) as usize;
        let d_start = b + d;
        let d_end = d_start + len;
        let s_start = b + s;
        let s_end = s_start + len;
        dest_expected[d_start..d_end].copy_from_slice(&src[s_start..s_end]);
        unsafe {
          copy_u8_backward(
            dest_actual.as_mut_ptr().add(d_start).add(len).cast(),
            src.as_ptr().add(s_start).add(len).cast(),
            len,
          )
        }
        assert_eq!(dest_expected, dest_actual);
      }
    }
  }

  // src == dest is allowed for simplicity (but has no effect)
  for len in 0..=16_usize {
    let base = rand_bytes(64);
    for s in 0..len {
      let mut new = base.clone();
      unsafe {
        let p = new.as_mut_ptr();
        copy_u8_backward(
          p.add(s).add(len).cast(),
          p.add(s).add(len).cast(),
          len,
        )
      }
      assert_eq!(base, new);
    }
  }

  // src > dest works even when the regions overlap
  for len in 0..=16_usize {
    let src = rand_bytes(64);
    for s in 0..len {
      for d in 0..len {
        if s > d {
          continue;
        }
        let mut out_expected = src.clone();
        let mut out_actual = src.clone();
        let b = (lcg.next_u32() % 16) as usize;
        let d_start = b + d;
        let s_start = b + s;
        let s_end = s_start + len;
        out_expected.copy_within(s_start..s_end, d_start);
        unsafe {
          let p = out_actual.as_mut_ptr();
          copy_u8_backward(
            p.add(d_start).add(len).cast(),
            p.add(s_start).add(len).cast(),
            len,
          )
        }
        assert_eq!(out_expected, out_actual);
      }
    }
  }
}

#[test]
fn test_copy_u16_forward() {
  let mut lcg = Lcg::new();

  // disjoint regions is the basic use case
  for len in 0..=16_usize {
    let src = rand_halfwords(64);
    for s in 0..len {
      for d in 0..len {
        let mut dest_expected = vec![0_u16; 64];
        let mut dest_actual = vec![0_u16; 64];
        let b = (lcg.next_u32() % 16) as usize;
        let d_start = b + d;
        let s_start = b + s;
        // assume `copy_u8_forward` already works and build our "expected" value
        // using it, which allows us to do copies of odd lengths.
        unsafe {
          copy_u8_forward(
            dest_expected.as_mut_ptr().add(d_start).cast(),
            src.as_ptr().add(s_start).cast(),
            len,
          )
        }
        unsafe {
          copy_u16_forward(
            dest_actual.as_mut_ptr().add(d_start).cast(),
            src.as_ptr().add(s_start).cast(),
            len,
          )
        }
        assert_eq!(dest_expected, dest_actual, "len:{len}");
      }
    }
  }

  // src == dest is allowed for simplicity (but has no effect)
  for len in 0..=16_usize {
    let base = rand_halfwords(64);
    for s in 0..len {
      let mut new = base.clone();
      unsafe {
        let p = new.as_mut_ptr();
        copy_u16_forward(p.add(s).cast(), p.add(s).cast(), len)
      }
      assert_eq!(base, new);
    }
  }

  // src > dest works even when the regions overlap
  for len in 3..=16_usize {
    let src = rand_halfwords(64);
    for s in 0..len {
      for d in 0..len {
        if s < d {
          continue;
        }
        let mut out_expected = src.clone();
        let mut out_actual = src.clone();
        let b = (lcg.next_u32() % 16) as usize;
        let d_start = b + d;
        let s_start = b + s;
        // assume `copy_u8_forward` already works and build our "expected" value
        // using it, which allows us to do copies of odd lengths.
        unsafe {
          copy_u8_forward(
            out_expected.as_mut_ptr().add(d_start).cast(),
            src.as_ptr().add(s_start).cast(),
            len,
          )
        }
        unsafe {
          let p = out_actual.as_mut_ptr();
          copy_u16_forward(p.add(d_start).cast(), p.add(s_start).cast(), len)
        }
        assert_eq!(out_expected, out_actual);
      }
    }
  }
}

#[test]
fn test_copy_u16_backward() {
  let mut lcg = Lcg::new();

  // disjoint regions is the basic use case
  for len in 0..=16_usize {
    let src = rand_halfwords(64);
    for s in 0..len {
      for d in 0..len {
        let mut dest_expected = vec![0_u16; 64];
        let mut dest_actual = vec![0_u16; 64];
        let b = (lcg.next_u32() % 16) as usize;
        let d_start = b + d;
        let s_start = b + s;
        // assume `copy_u8_backward` works and use it so we can have odd lengths
        unsafe {
          copy_u8_backward(
            dest_expected.as_mut_ptr().add(48).sub(d_start).cast(),
            src.as_ptr().add(48).sub(s_start).cast(),
            len,
          )
        }
        unsafe {
          copy_u16_backward(
            dest_actual.as_mut_ptr().add(48).sub(d_start).cast(),
            src.as_ptr().add(48).sub(s_start).cast(),
            len,
          )
        }
        assert_eq!(dest_expected, dest_actual);
      }
    }
  }

  // src == dest is allowed for simplicity (but has no effect)
  for len in 0..=16_usize {
    let base = rand_halfwords(64);
    for s in 0..len {
      let mut new = base.clone();
      unsafe {
        let p = new.as_mut_ptr();
        copy_u16_backward(p.add(48).sub(s).cast(), p.add(48).sub(s).cast(), len)
      }
      assert_eq!(base, new);
    }
  }

  // src > dest works even when the regions overlap
  for len in 0..=16_usize {
    let src = rand_halfwords(64);
    for s in 0..len {
      for d in 0..len {
        if s > d {
          continue;
        }
        let mut out_expected = src.clone();
        let mut out_actual = src.clone();
        let b = (lcg.next_u32() % 16) as usize;
        let d_start = b + d;
        let s_start = b + s;
        // assume `copy_u8_backward` works and use it so we can have odd lengths
        unsafe {
          let p = out_expected.as_mut_ptr();
          copy_u8_backward(
            p.add(48).sub(d_start).cast(),
            p.add(48).sub(s_start).cast(),
            len,
          )
        }
        unsafe {
          let p = out_actual.as_mut_ptr();
          copy_u16_backward(
            p.add(48).sub(d_start).cast(),
            p.add(48).sub(s_start).cast(),
            len,
          )
        }
        assert_eq!(out_expected, out_actual);
      }
    }
  }
}

#[test]
fn test_copy_u32_forward() {
  let mut lcg = Lcg::new();

  // disjoint regions is the basic use case
  for len in 0..=16_usize {
    let src = rand_words(64);
    for s in 0..len {
      for d in 0..len {
        let mut dest_expected = vec![0_u32; 64];
        let mut dest_actual = vec![0_u32; 64];
        let b = (lcg.next_u32() % 16) as usize;
        let d_start = b + d;
        let s_start = b + s;
        // assume `copy_u16_forward` already works and build our "expected"
        // value using it, which allows us to do copies of any length.
        unsafe {
          copy_u8_forward(
            dest_expected.as_mut_ptr().add(d_start).cast(),
            src.as_ptr().add(s_start).cast(),
            len,
          )
        }
        unsafe {
          copy_u32_forward(
            dest_actual.as_mut_ptr().add(d_start).cast(),
            src.as_ptr().add(s_start).cast(),
            len,
          )
        }
        assert_eq!(dest_expected, dest_actual, "len:{len}");
      }
    }
  }

  // src == dest is allowed for simplicity (but has no effect)
  for len in 0..=16_usize {
    let base = rand_words(64);
    for s in 0..len {
      let mut new = base.clone();
      unsafe {
        let p = new.as_mut_ptr();
        copy_u16_forward(p.add(s).cast(), p.add(s).cast(), len)
      }
      assert_eq!(base, new);
    }
  }

  // src > dest works even when the regions overlap
  for len in 3..=16_usize {
    let src = rand_words(64);
    for s in 0..len {
      for d in 0..len {
        if s < d {
          continue;
        }
        let mut out_expected = src.clone();
        let mut out_actual = src.clone();
        let b = (lcg.next_u32() % 16) as usize;
        let d_start = b + d;
        let s_start = b + s;
        // assume `copy_u8_forward` already works and build our "expected" value
        // using it, which allows us to do copies of any length.
        unsafe {
          copy_u8_forward(
            out_expected.as_mut_ptr().add(d_start).cast(),
            src.as_ptr().add(s_start).cast(),
            len,
          )
        }
        unsafe {
          let p = out_actual.as_mut_ptr();
          copy_u32_forward(p.add(d_start).cast(), p.add(s_start).cast(), len)
        }
        assert_eq!(out_expected, out_actual);
      }
    }
  }
}

#[test]
fn test_copy_u32_backward() {
  let mut lcg = Lcg::new();

  // disjoint regions is the basic use case
  for len in 0..=16_usize {
    let src = rand_words(64);
    for s in 0..len {
      for d in 0..len {
        let mut dest_expected = vec![0_u16; 64];
        let mut dest_actual = vec![0_u16; 64];
        let b = (lcg.next_u32() % 16) as usize;
        let d_start = b + d;
        let s_start = b + s;
        // assume `copy_u8_backward` works and use it so we can have odd lengths
        unsafe {
          copy_u8_backward(
            dest_expected.as_mut_ptr().add(48).sub(d_start).cast(),
            src.as_ptr().add(48).sub(s_start).cast(),
            len,
          )
        }
        unsafe {
          copy_u16_backward(
            dest_actual.as_mut_ptr().add(48).sub(d_start).cast(),
            src.as_ptr().add(48).sub(s_start).cast(),
            len,
          )
        }
        assert_eq!(dest_expected, dest_actual);
      }
    }
  }

  // src == dest is allowed for simplicity (but has no effect)
  for len in 0..=16_usize {
    let base = rand_words(64);
    for s in 0..len {
      let mut new = base.clone();
      unsafe {
        let p = new.as_mut_ptr();
        copy_u32_backward(p.add(48).sub(s).cast(), p.add(48).sub(s).cast(), len)
      }
      assert_eq!(base, new);
    }
  }

  // src > dest works even when the regions overlap
  for len in 0..=16_usize {
    let src = rand_words(64);
    for s in 0..len {
      for d in 0..len {
        if s > d {
          continue;
        }
        let mut out_expected = src.clone();
        let mut out_actual = src.clone();
        let b = (lcg.next_u32() % 16) as usize;
        let d_start = b + d;
        let s_start = b + s;
        // assume `copy_u8_backward` works and use it so we can have odd lengths
        unsafe {
          let p = out_expected.as_mut_ptr();
          copy_u8_backward(
            p.add(48).sub(d_start).cast(),
            p.add(48).sub(s_start).cast(),
            len,
          )
        }
        unsafe {
          let p = out_actual.as_mut_ptr();
          copy_u32_backward(
            p.add(48).sub(d_start).cast(),
            p.add(48).sub(s_start).cast(),
            len,
          )
        }
        assert_eq!(out_expected, out_actual);
      }
    }
  }
}
