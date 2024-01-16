use std::ffi::{CString, NulError};

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
impl ToLength for &str {
    fn to_length(&self) -> usize {
        CString::new(*self).unwrap().count_bytes()
    }
}
impl ToLength for String {
    fn to_length(&self) -> usize {
        CString::new(self.as_bytes()).unwrap().count_bytes()
    }
}
impl ToLength for CString {
    fn to_length(&self) -> usize {
        self.as_bytes().len()
    }
}

pub trait ToCstring {
    fn to_cstring(&self) -> Result<CString, NulError>;
}

impl ToCstring for &str {
    fn to_cstring(&self) -> Result<CString, NulError> {
        Ok(CString::new(*self)?)
    }
}
impl ToCstring for String {
    fn to_cstring(&self) -> Result<CString, NulError> {
        Ok(CString::new(self.as_bytes())?)
    }
}