use crate::record_collection::RecordErr;

pub fn bytes_to_trimmed_string(bytes: &[u8]) -> Result<String, ()> {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    std::str::from_utf8(&bytes[..end])
        .map_err(|_| ())
        .map(|s| s.to_string())
}
