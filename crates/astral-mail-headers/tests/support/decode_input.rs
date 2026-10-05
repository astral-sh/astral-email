/// Keep arbitrary bytes in one header value without splitting CRLF pairs.
pub(crate) fn wrap(bytes: &[u8]) -> Vec<u8> {
    let mut source = Vec::with_capacity(bytes.len() * 2 + 5);
    source.extend_from_slice(b"X: ");
    for (index, &byte) in bytes.iter().enumerate() {
        source.push(byte);
        if byte == b'\n' || (byte == b'\r' && bytes.get(index + 1) != Some(&b'\n')) {
            source.push(b' ');
        }
    }
    source.extend_from_slice(b"\n\n");
    source
}
