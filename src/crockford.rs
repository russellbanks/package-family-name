/// Crockford Base32 encode an 8-byte array.
pub fn encode_lower(input: [u8; 8]) -> [u8; 13] {
    const CROCKFORD_LOWER: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";

    let mut out = [0; 13];

    let n = u64::from_be_bytes(input);

    for (index, shift) in (4..=59).rev().step_by(5).enumerate() {
        out[index] = CROCKFORD_LOWER[((n >> shift) & 0x1F) as usize];
    }

    // Final character encodes the remaining 4 bits + 1 zero padding bit
    out[12] = CROCKFORD_LOWER[((n << 1) & 0x1F) as usize];

    out
}
