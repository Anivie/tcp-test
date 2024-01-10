use std::ffi::CString;

pub trait ToAddress {
    fn to_address(&self) -> Option<(u16, &str)>;
}

impl ToAddress for &str {
    fn to_address(&self) -> Option<(u16, &str)> {
        let parts: Vec<&str> = self.split(':').collect();
        if parts.is_empty() { return None; }
        if parts.len() == 1 { return None; }

        let addr = parts[0];
        let port = parts[1].parse::<u16>().unwrap();
        Some((port, addr))
    }
}

pub trait ToLength {
    fn to_length(&self) -> usize;
}

pub trait ToData {
    fn to_data(&self) -> &[u8];
}

impl ToLength for &str {
    fn to_length(&self) -> usize {
        self.len()
    }
}

impl ToData for &str {
    fn to_data(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ToLength for String {
    fn to_length(&self) -> usize {
        self.len()
    }
}

impl ToData for String {
    fn to_data(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ToLength for CString {
    fn to_length(&self) -> usize {
        self.as_bytes().len()
    }
}

impl ToData for CString {
    fn to_data(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ToLength for Vec<u8> {
    fn to_length(&self) -> usize {
        self.len()
    }
}

impl ToData for Vec<u8> {
    fn to_data(&self) -> &[u8] {
        self.as_slice()
    }
}
