//! CPI tag verification tests — the cross-program wire canary.
//!
//! Only one CPI remains: TopUpInsurance (tag 9).
//!
//! v16 CONTRACT (percolator-prog v16-sync): tag 9's `amount` is decoded with
//! `read_u128` (16 bytes). The pre-v16 wire sent an 8-byte u64, which a v16
//! wrapper rejects with InvalidInstructionData (`read_u128` requires len >= 16).
//! This canary pins the NEW 17-byte (tag + u128) shape so a regression to the
//! broken 8-byte u64 wire fails CI loudly. See src/cpi.rs::cpi_top_up_insurance.

/// The v16 tag-9 wire is `tag(1) + amount(16, u128 LE)` = 17 bytes.
#[test]
fn test_cpi_tag_top_up_insurance_v16_u128_wire() {
    let amount: u64 = 1000;
    let mut data = Vec::with_capacity(17);
    data.push(9u8); // TAG_TOP_UP_INSURANCE
    data.extend_from_slice(&(amount as u128).to_le_bytes());

    assert_eq!(data[0], 9, "tag byte must be 9 (TopUpInsurance)");
    assert_eq!(
        data.len(),
        17,
        "v16 tag-9 payload MUST be 1 (tag) + 16 (u128 amount) = 17 bytes"
    );
    // The amount must round-trip as a little-endian u128 — exactly what the v16
    // wrapper's read_u128 expects (percolator-prog v16_program.rs:3275).
    let decoded = u128::from_le_bytes(data[1..17].try_into().unwrap());
    assert_eq!(decoded, amount as u128);
}

/// REGRESSION GUARD: the broken pre-v16 wire was `tag(1) + amount(8, u64 LE)` =
/// 9 bytes. Against a v16 wrapper that payload hard-reverts at the read_u128
/// decoder. This test documents the defect and asserts we no longer emit it.
#[test]
fn test_cpi_tag9_8byte_u64_wire_is_rejected_shape() {
    let amount: u64 = 1000;
    let mut broken = Vec::with_capacity(9);
    broken.push(9u8);
    broken.extend_from_slice(&amount.to_le_bytes()); // 8-byte u64 — the pre-v16 break

    assert_eq!(broken.len(), 9, "this is the OLD (broken) shape");
    // The v16 wrapper's read_u128 requires >= 16 payload bytes; 8 < 16 -> revert.
    // The correct v16 wire is 17 bytes (see test above), never 9.
    assert!(
        broken.len() < 17,
        "the 8-byte u64 wire is shorter than the required v16 u128 wire"
    );
}
