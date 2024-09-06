#[inline]
pub(crate) fn find_nl(s: &[u8]) -> usize {
    let mut i = 0;
    while i < s.len() {
        if s[i] == b'\n' || s[i] == b'\r' {
            break;
        }
        i += 1;
    }
    // SAFETY: This assert won't fail if the code above is correct
    unsafe_assert!(i <= s.len());
    i
}

#[inline]
pub(crate) fn find_nl_chr(s: &[u8], chr: u8) -> usize {
    let mut i = 0;
    while i < s.len() {
        if s[i] == b'\n' || s[i] == b'\r' || s[i] == chr {
            break;
        }
        i += 1;
    }
    // SAFETY: This assert won't fail if the code above is correct
    unsafe_assert!(i <= s.len());
    i
}
