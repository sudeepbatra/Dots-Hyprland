// Take a look at the license at the top of the repository in the LICENSE file.

use crate::error::Error;
use std::ffi::CStr;
use std::fmt;

#[doc(alias = "cairo_debug_reset_static_data")]
pub unsafe fn debug_reset_static_data() {
    ffi::cairo_debug_reset_static_data()
}

pub fn status_to_result(status: ffi::cairo_status_t) -> Result<(), Error> {
    match status {
        ffi::STATUS_SUCCESS => Ok(()),
        err => Err(err.into()),
    }
}

#[doc(alias = "cairo_version_string")]
#[doc(alias = "get_version_string")]
pub fn version_string() -> &'static str {
    unsafe {
        let ptr = ffi::cairo_version_string();
        CStr::from_ptr(ptr)
            .to_str()
            .expect("invalid version string")
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub micro: u8,
}

impl Version {
    #[allow(clippy::self_named_constructors)]
    #[doc(alias = "cairo_version")]
    #[doc(alias = "get_version")]
    pub fn version() -> Version {
        let version = unsafe { ffi::cairo_version() };
        Version {
            major: (version / 10_000 % 100) as _,
            minor: (version / 100 % 100) as _,
            micro: (version % 100) as _,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.micro)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_versions() {
        assert_eq!(version_string(), Version::version().to_string());
    }
}
