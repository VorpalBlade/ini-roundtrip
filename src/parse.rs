/*!
Optimized routines for parsing INI.

This module provides 2 functions: `find_nl` and `find_nl_chr`:

* `fn find_nl(s: &[u8]) -> usize`

  Finds the first `b'\r'` or `b'\n'` in the input byte string and returns its index.
  If no match was found returns the length of the input.

* `fn find_nl_chr(s: &[u8], chr: u8) -> usize`

  Finds the first `b'\r'`, `b'\n'` or `chr` in the input byte string and returns its index.
  If no match was found returns the length of the input.

For more information on the SWAR approaches see: <http://0x80.pl/articles/simd-strfind.html#swar>.
In reality, I only see minor improvements with SWAR (about 33% faster).

*/

#[cfg(not(debug_assertions))]
/// This macro will use `unreachable_unchecked` in release mode, but check the
/// condition in debug mode.
macro_rules! unsafe_assert {
    ($e:expr) => {
        // SAFETY: The safety comment is documented where the macro is used (that is why
        // it has "unsafe" in the name)
        unsafe {
            if !$e {
                ::core::hint::unreachable_unchecked();
            }
        }
    };
}
#[cfg(debug_assertions)]
/// This macro will use `unreachable_unchecked` in release mode, but check the
/// condition in debug mode.
macro_rules! unsafe_assert {
    ($e:expr_2021) => {
        if !$e {
            panic!("assertion failed: {}", stringify!($e));
        }
    };
}

mod generic;

cfg_if::cfg_if! {
    // These optimizations are little endian specific
    if #[cfg(not(target_endian = "little"))] {
        pub(crate) use self::generic::*;
    }
    else if #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "avx2"))] {
        mod avx2;
        pub(crate) use self::avx2::*;
    }
    else if #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_feature = "sse2"))] {
        mod sse2;
        pub(crate) use self::sse2::*;
    }
    else if #[cfg(target_pointer_width = "64")] {
        mod swar64;
        pub(crate) use self::swar64::*;
    }
    else if #[cfg(target_pointer_width = "32")] {
        mod swar32;
        pub(crate) use self::swar32::*;
    }
    else {
        pub(crate) use self::generic::*;
    }
}

#[test]
fn test_parse() {
    let mut buffer = [b'-'; 254];
    for i in 0..buffer.len() {
        buffer[i] = b'\n';

        // Check reference implementation
        assert_eq!(generic::find_nl(&buffer), i);
        assert_eq!(generic::find_nl_chr(&buffer, b'='), i);

        // Check target implementation
        assert_eq!(find_nl(&buffer), i);
        assert_eq!(find_nl_chr(&buffer, b'='), i);

        // Write annoying byte back
        buffer[i] = if i & 1 == 0 { !0x0D } else { !0x0A };
    }
}
