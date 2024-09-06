#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::_mm256_cmpeq_epi8;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::_mm256_lddqu_si256;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::_mm256_movemask_epi8;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::_mm256_or_si256;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::_mm256_set1_epi8;

#[inline]
pub(crate) fn find_nl(s: &[u8]) -> usize {
    let mut offset = 0;

    // SAFETY:
    // * We don't build this entire module if we don't have AVX2 (see parse.rs)
    // * The while condition ensures the pointer is in bounds.
    // * The load uses a variant that allows for unaligned loads (so that is safe).
    unsafe {
        let n_lit = _mm256_set1_epi8(b'\n' as i8);
        let r_lit = _mm256_set1_epi8(b'\r' as i8);

        while offset + 32 <= s.len() {
            let block = _mm256_lddqu_si256(s.as_ptr().add(offset).cast());

            let n_eq = _mm256_cmpeq_epi8(n_lit, block);
            let r_eq = _mm256_cmpeq_epi8(r_lit, block);

            let mask = _mm256_movemask_epi8(_mm256_or_si256(n_eq, r_eq));

            if mask != 0 {
                return offset + mask.trailing_zeros() as usize;
            }

            offset += 32;
        }
    }

    // SAFETY: This assert won't fail if the code above is correct
    unsafe_assert!(offset <= s.len());
    offset += super::generic::find_nl(&s[offset..]);
    // SAFETY: This assert won't fail if find_nl is correct (which we assume)
    unsafe_assert!(offset <= s.len());
    offset
}

#[inline]
pub(crate) fn find_nl_chr(s: &[u8], chr: u8) -> usize {
    let mut offset = 0;

    // SAFETY:
    // * We don't build this entire module if we don't have AVX2 (see parse.rs)
    // * The while condition ensures the pointer is in bounds.
    // * The load uses a variant that allows for unaligned loads (so that is safe).
    unsafe {
        let n_lit = _mm256_set1_epi8(b'\n' as i8);
        let r_lit = _mm256_set1_epi8(b'\r' as i8);
        let c_lit = _mm256_set1_epi8(chr as i8);

        while offset + 32 <= s.len() {
            let block = _mm256_lddqu_si256(s.as_ptr().add(offset).cast());

            let n_eq = _mm256_cmpeq_epi8(n_lit, block);
            let r_eq = _mm256_cmpeq_epi8(r_lit, block);
            let c_eq = _mm256_cmpeq_epi8(c_lit, block);

            let mask = _mm256_movemask_epi8(_mm256_or_si256(_mm256_or_si256(n_eq, r_eq), c_eq));

            if mask != 0 {
                return offset + mask.trailing_zeros() as usize;
            }

            offset += 32;
        }
    }

    // SAFETY: This assert won't fail if the code above is correct
    unsafe_assert!(offset <= s.len());
    offset += super::generic::find_nl_chr(&s[offset..], chr);
    // SAFETY: This assert won't fail if find_nl_chr is correct (which we assume)
    unsafe_assert!(offset <= s.len());
    offset
}
