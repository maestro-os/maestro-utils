//! TODO doc

// TODO

/// Computes the lookup table for the given generator polynomial.
///
/// Arguments:
/// - `table` is filled with the table's values.
/// - `polynom` is the polynom.
pub fn compute_lookuptable(table: &mut [u32; 256], polynom: u32) {
    // Little endian
    let mut i = table.len() / 2;
    let mut crc = 1;

    while i > 0 {
        if crc & 1 != 0 {
            crc = (crc >> 1) ^ polynom;
        } else {
            crc >>= 1;
        }

        for j in (0..table.len()).step_by(2 * i) {
            table[i ^ j] = crc ^ table[j];
        }

        i >>= 1;
    }
}

/// Computes the CRC32 checksum on the given data `data` with the given table
/// `table` for the wanted generator polynomial.
pub fn compute(data: &[u8], table: &[u32; 256]) -> u32 {
    // Sarwate algorithm
    let mut crc = !0u32;

    for b in data {
        let i = ((crc as usize) ^ (*b as usize)) & 0xff;
        crc = table[i] ^ (crc >> 8);
    }

    !crc
}

// TODO Test CRC32
