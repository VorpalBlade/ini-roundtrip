#[inline]
pub(crate) fn find_nl(s: &[u8]) -> usize {
    let mut offset = 0;

    let n_lit = b'\n' as u32 * 0x01010101u32;
    let r_lit = b'\r' as u32 * 0x01010101u32;
    while offset + 4 <= s.len() {
        // SAFETY:
        // * The pointer is in bounds by the condition in the while loop
        // * We use read_unaligned, so alignment is not a concern
        let word = unsafe { s.as_ptr().add(offset).cast::<u32>().read_unaligned() };
        let mask = cmpeq(n_lit, word) | cmpeq(r_lit, word);
        if mask != 0 {
            return offset + (mask.trailing_zeros() >> 3) as usize;
        }

        offset += 4;
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

    let n_lit = b'\n' as u32 * 0x01010101u32;
    let r_lit = b'\r' as u32 * 0x01010101u32;
    let c_lit = chr as u32 * 0x01010101u32;
    while offset + 4 <= s.len() {
        // SAFETY:
        // * The pointer is in bounds by the condition in the while loop
        // * We use read_unaligned, so alignment is not a concern
        let word = unsafe { s.as_ptr().add(offset).cast::<u32>().read_unaligned() };
        let mask = cmpeq(n_lit, word) | cmpeq(r_lit, word) | cmpeq(c_lit, word);
        if mask != 0 {
            return offset + (mask.trailing_zeros() >> 3) as usize;
        }

        offset += 4;
    }

    // SAFETY: This assert won't fail if the code above is correct
    unsafe_assert!(offset <= s.len());
    offset += super::generic::find_nl_chr(&s[offset..], chr);
    // SAFETY: This assert won't fail if find_nl is correct (which we assume)
    unsafe_assert!(offset <= s.len());
    offset
}

#[inline]
fn cmpeq(needle: u32, haystack: u32) -> u32 {
    let neq = !(needle ^ haystack);
    let t0 = (neq & 0x7f7f7f7f) + 0x01010101;
    let t1 = neq & 0x80808080;
    t0 & t1
}
