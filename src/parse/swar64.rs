#[inline]
pub(crate) fn find_nl(s: &[u8]) -> usize {
    let mut offset = 0;

    let n_lit = b'\n' as u64 * 0x0101010101010101u64;
    let r_lit = b'\r' as u64 * 0x0101010101010101u64;
    while offset + 8 <= s.len() {
        // SAFETY:
        // * The pointer is in bounds by the condition in the while loop
        // * We use read_unaligned, so alignment is not a concern
        let word = unsafe { s.as_ptr().add(offset).cast::<u64>().read_unaligned() };
        let mask = cmpeq(n_lit, word) | cmpeq(r_lit, word);
        if mask != 0 {
            return offset + (mask.trailing_zeros() >> 3) as usize;
        }

        offset += 8;
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

    let n_lit = b'\n' as u64 * 0x0101010101010101u64;
    let r_lit = b'\r' as u64 * 0x0101010101010101u64;
    let c_lit = chr as u64 * 0x0101010101010101u64;
    while offset + 8 <= s.len() {
        // SAFETY:
        // * The pointer is in bounds by the condition in the while loop
        // * We use read_unaligned, so alignment is not a concern
        let word = unsafe { s.as_ptr().add(offset).cast::<u64>().read_unaligned() };
        let mask = cmpeq(n_lit, word) | cmpeq(r_lit, word) | cmpeq(c_lit, word);
        if mask != 0 {
            return offset + (mask.trailing_zeros() >> 3) as usize;
        }

        offset += 8;
    }

    // SAFETY: This assert won't fail if the code above is correct
    unsafe_assert!(offset <= s.len());
    offset += super::generic::find_nl_chr(&s[offset..], chr);
    // SAFETY: This assert won't fail if find_nl is correct (which we assume)
    unsafe_assert!(offset <= s.len());
    offset
}

#[inline]
fn cmpeq(needle: u64, haystack: u64) -> u64 {
    let neq = !(needle ^ haystack);
    let t0 = (neq & 0x7f7f7f7f7f7f7f7f) + 0x0101010101010101;
    let t1 = neq & 0x8080808080808080;
    t0 & t1
}
