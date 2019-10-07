use std::fmt;
use hex::ToHex;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(transparent)]
pub struct MD5(#[serde(with = "hex_serde")] pub [u8; 16]);

impl fmt::Display for MD5 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self)
    }
}

impl fmt::UpperHex for MD5 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = self.0;
        write!(f, "{}", v.encode_hex_upper::<String>())
    }
}

impl fmt::LowerHex for MD5 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = self.0;
        write!(f, "{}", v.encode_hex::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex::FromHex;

    #[test]
    fn formats_md5_as_lower_hex() {
        let hex_string = "f74caae806e2a5f62828c1db0a01e8d2";
        let md5 = MD5(<[u8; 16]>::from_hex(hex_string).unwrap());

        assert_eq!(format!("{}", md5), hex_string.to_lowercase());
    }

    #[test]
    fn can_format_md5_as_lower_hex_explicit() {
        let hex_string = "f74caae806e2a5f62828c1db0a01e8d2";
        let md5 = MD5(<[u8; 16]>::from_hex(hex_string).unwrap());

        assert_eq!(format!("{:x}", md5), hex_string.to_lowercase());
    }

    #[test]
    fn can_format_md5_as_uppper_hex_explicit() {
        let hex_string = "f74caae806e2a5f62828c1db0a01e8d2";
        let md5 = MD5(<[u8; 16]>::from_hex(hex_string).unwrap());

        assert_eq!(format!("{:X}", md5), hex_string.to_uppercase());
    }
}
