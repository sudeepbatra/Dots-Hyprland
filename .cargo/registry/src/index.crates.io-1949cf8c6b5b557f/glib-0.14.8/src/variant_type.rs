// Take a look at the license at the top of the repository in the LICENSE file.

use crate::translate::*;
use crate::types::StaticType;
use crate::types::Type;
use crate::BoolError;
use std::borrow::{Borrow, Cow, ToOwned};
use std::cmp::{Eq, PartialEq};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::slice;

/// Describes `Variant` types.
///
/// The `Variant` type system (based on the D-Bus one) describes types with
/// "type strings". `VariantType` is an owned immutable type string (you can
/// think of it as a `Box<str>` statically guaranteed to be a valid type
/// string), `&VariantTy` is a borrowed one (like `&str`).
#[doc(alias = "GVariantType")]
pub struct VariantType {
    // GVariantType* essentially is a char*, that always is valid UTF-8 but
    // isn't NUL-terminated.
    ptr: *mut ffi::GVariantType,
    // We query the length on creation assuming it's cheap (because type strings
    // are short) and likely to happen anyway.
    len: usize,
}

impl VariantType {
    /// Tries to create a `VariantType` from a string slice.
    ///
    /// Returns `Ok` if the string is a valid type string, `Err` otherwise.
    pub fn new(type_string: &str) -> Result<VariantType, BoolError> {
        VariantTy::new(type_string).map(ToOwned::to_owned)
    }
}

unsafe impl Send for VariantType {}
unsafe impl Sync for VariantType {}

impl Drop for VariantType {
    fn drop(&mut self) {
        unsafe { ffi::g_variant_type_free(self.ptr) }
    }
}

impl Borrow<VariantTy> for VariantType {
    fn borrow(&self) -> &VariantTy {
        self
    }
}

impl Clone for VariantType {
    fn clone(&self) -> VariantType {
        unsafe {
            VariantType {
                ptr: ffi::g_variant_type_copy(self.ptr),
                len: self.len,
            }
        }
    }
}

impl Deref for VariantType {
    type Target = VariantTy;
    fn deref(&self) -> &VariantTy {
        unsafe {
            &*(slice::from_raw_parts(self.ptr as *const u8, self.len) as *const [u8]
                as *const VariantTy)
        }
    }
}

impl fmt::Debug for VariantType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <VariantTy as fmt::Debug>::fmt(self, f)
    }
}

impl fmt::Display for VariantType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

impl Hash for VariantType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        <VariantTy as Hash>::hash(self, state)
    }
}

impl<'a> From<VariantType> for Cow<'a, VariantTy> {
    fn from(ty: VariantType) -> Cow<'a, VariantTy> {
        Cow::Owned(ty)
    }
}

#[doc(hidden)]
impl<'a> ToGlibPtr<'a, *const ffi::GVariantType> for VariantType {
    type Storage = &'a Self;

    fn to_glib_none(&'a self) -> Stash<'a, *const ffi::GVariantType, Self> {
        Stash(self.ptr, self)
    }

    fn to_glib_full(&self) -> *const ffi::GVariantType {
        unsafe { ffi::g_variant_type_copy(self.ptr) }
    }
}

#[doc(hidden)]
impl<'a> ToGlibPtr<'a, *mut ffi::GVariantType> for VariantType {
    type Storage = &'a Self;

    fn to_glib_none(&'a self) -> Stash<'a, *mut ffi::GVariantType, Self> {
        Stash(self.ptr, self)
    }

    fn to_glib_full(&self) -> *mut ffi::GVariantType {
        unsafe { ffi::g_variant_type_copy(self.ptr) }
    }
}

#[doc(hidden)]
impl<'a> ToGlibPtrMut<'a, *mut ffi::GVariantType> for VariantType {
    type Storage = &'a mut Self;

    fn to_glib_none_mut(&'a mut self) -> StashMut<'a, *mut ffi::GVariantType, Self> {
        StashMut(self.ptr, self)
    }
}

#[doc(hidden)]
impl FromGlibPtrNone<*const ffi::GVariantType> for VariantType {
    unsafe fn from_glib_none(ptr: *const ffi::GVariantType) -> VariantType {
        VariantTy::from_ptr(ptr).to_owned()
    }
}

#[doc(hidden)]
impl FromGlibPtrFull<*const ffi::GVariantType> for VariantType {
    unsafe fn from_glib_full(ptr: *const ffi::GVariantType) -> VariantType {
        // Don't assume ownership of a const pointer.
        // A transfer: full annotation on a `const GVariantType*` is likely a bug.
        VariantTy::from_ptr(ptr).to_owned()
    }
}

/// Describes `Variant` types.
///
/// This is a borrowed counterpart of [`VariantType`](struct.VariantType.html).
/// Essentially it's a `str` statically guaranteed to be a valid type string.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct VariantTy {
    inner: str,
}

impl VariantTy {
    /// An indefinite type that is a supertype of every type (including itself).
    #[doc(alias = "G_VARIANT_TYPE_ANY")]
    pub const ANY: &'static VariantTy = unsafe { std::mem::transmute::<&str, &VariantTy>("*") };

    /// Tries to create a `&VariantTy` from a string slice.
    ///
    /// Returns `Ok` if the string is a valid type string, `Err` otherwise.
    pub fn new(type_string: &str) -> Result<&VariantTy, BoolError> {
        let ptr = type_string.as_ptr();
        let limit = ptr as usize + type_string.len();
        let mut end = 0_usize;
        unsafe {
            let ok = from_glib(ffi::g_variant_type_string_scan(
                ptr as *const _,
                limit as *const _,
                &mut end as *mut usize as *mut _,
            ));
            if ok && end == limit {
                Ok(&*(type_string.as_bytes() as *const [u8] as *const VariantTy))
            } else {
                Err(bool_error!("Invalid type string: '{}'", type_string))
            }
        }
    }

    /// Converts a type string into `&VariantTy` without any checks.
    ///
    /// # Safety
    ///
    /// The caller is responsible for passing in only a valid variant type string
    /// which is already registered with the type system.
    pub unsafe fn from_str_unchecked(type_string: &str) -> &VariantTy {
        &*(type_string as *const str as *const VariantTy)
    }

    /// Creates `&VariantTy` with a wildcard lifetime from a `GVariantType`
    /// pointer.
    #[doc(hidden)]
    pub unsafe fn from_ptr<'a>(ptr: *const ffi::GVariantType) -> &'a VariantTy {
        let len = ffi::g_variant_type_get_string_length(ptr) as usize;
        &*(slice::from_raw_parts(ptr as *const u8, len) as *const [u8] as *const VariantTy)
    }

    /// Returns a `GVariantType` pointer.
    #[doc(hidden)]
    pub fn as_ptr(&self) -> *const ffi::GVariantType {
        self.inner.as_ptr() as *const _
    }

    /// Converts to a string slice.
    pub fn to_str(&self) -> &str {
        &self.inner
    }

    /// Return this type as an array.
    pub(crate) fn with_array(&self) -> VariantType {
        VariantType::new(&format!("a{}", self.to_str())).expect("invalid variant signature")
    }
}

unsafe impl Sync for VariantTy {}

#[doc(hidden)]
impl<'a> ToGlibPtr<'a, *const ffi::GVariantType> for VariantTy {
    type Storage = &'a Self;

    fn to_glib_none(&'a self) -> Stash<'a, *const ffi::GVariantType, Self> {
        Stash(self.as_ptr(), self)
    }
}

impl fmt::Display for VariantTy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

impl<'a> From<&'a VariantTy> for Cow<'a, VariantTy> {
    fn from(ty: &'a VariantTy) -> Cow<'a, VariantTy> {
        Cow::Borrowed(ty)
    }
}

impl ToOwned for VariantTy {
    type Owned = VariantType;

    fn to_owned(&self) -> VariantType {
        unsafe {
            VariantType {
                ptr: ffi::g_variant_type_copy(self.as_ptr()),
                len: self.inner.len(),
            }
        }
    }
}

impl StaticType for VariantTy {
    fn static_type() -> Type {
        unsafe { from_glib(ffi::g_variant_type_get_gtype()) }
    }
}

#[doc(hidden)]
unsafe impl<'a> crate::value::FromValue<'a> for &'a VariantTy {
    type Checker = crate::value::GenericValueTypeOrNoneChecker<Self>;

    unsafe fn from_value(value: &'a crate::Value) -> Self {
        let ptr = gobject_ffi::g_value_get_boxed(value.to_glib_none().0);
        assert!(!ptr.is_null());
        VariantTy::from_ptr(ptr as *const ffi::GVariantType)
    }
}

#[doc(hidden)]
impl crate::value::ToValue for VariantTy {
    fn to_value(&self) -> crate::Value {
        unsafe {
            let mut value = crate::Value::from_type(VariantTy::static_type());
            gobject_ffi::g_value_set_boxed(
                value.to_glib_none_mut().0,
                self.to_glib_none().0 as *mut _,
            );
            value
        }
    }

    fn value_type(&self) -> crate::Type {
        VariantTy::static_type()
    }
}

#[doc(hidden)]
impl crate::value::ToValue for &VariantTy {
    fn to_value(&self) -> crate::Value {
        (*self).to_value()
    }

    fn value_type(&self) -> crate::Type {
        VariantTy::static_type()
    }
}

#[doc(hidden)]
impl crate::value::ToValueOptional for &VariantTy {
    fn to_value_optional(s: Option<&Self>) -> crate::Value {
        let mut value = crate::Value::for_value_type::<VariantType>();
        unsafe {
            gobject_ffi::g_value_set_boxed(
                value.to_glib_none_mut().0,
                s.to_glib_none().0 as *mut _,
            );
        }

        value
    }
}

impl StaticType for VariantType {
    fn static_type() -> Type {
        unsafe { from_glib(ffi::g_variant_type_get_gtype()) }
    }
}

#[doc(hidden)]
impl crate::value::ValueType for VariantType {
    type Type = VariantType;
}

#[doc(hidden)]
unsafe impl<'a> crate::value::FromValue<'a> for VariantType {
    type Checker = crate::value::GenericValueTypeOrNoneChecker<Self>;

    unsafe fn from_value(value: &'a crate::Value) -> Self {
        let ptr = gobject_ffi::g_value_get_boxed(value.to_glib_none().0);
        assert!(!ptr.is_null());
        from_glib_none(ptr as *const ffi::GVariantType)
    }
}

#[doc(hidden)]
impl crate::value::ToValue for VariantType {
    fn to_value(&self) -> crate::Value {
        unsafe {
            let mut value = crate::Value::from_type(VariantType::static_type());
            gobject_ffi::g_value_set_boxed(
                value.to_glib_none_mut().0,
                ToGlibPtr::<*mut _>::to_glib_none(&self).0 as *mut _,
            );
            value
        }
    }

    fn value_type(&self) -> crate::Type {
        VariantType::static_type()
    }
}

#[doc(hidden)]
impl crate::value::ToValueOptional for VariantType {
    fn to_value_optional(s: Option<&Self>) -> crate::Value {
        let mut value = crate::Value::for_value_type::<Self>();
        unsafe {
            gobject_ffi::g_value_set_boxed(
                value.to_glib_none_mut().0,
                ToGlibPtr::<*mut _>::to_glib_none(&s).0 as *mut _,
            );
        }

        value
    }
}

impl PartialEq for VariantType {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        <VariantTy as PartialEq>::eq(self, other)
    }
}

macro_rules! impl_eq {
    ($lhs:ty, $rhs: ty) => {
        impl<'a, 'b> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                <VariantTy as PartialEq>::eq(self, other)
            }
        }

        impl<'a, 'b> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                <VariantTy as PartialEq>::eq(self, other)
            }
        }
    };
}

impl_eq!(VariantType, VariantTy);
impl_eq!(VariantType, &'a VariantTy);
impl_eq!(VariantType, Cow<'a, VariantTy>);
impl_eq!(&'a VariantTy, Cow<'b, VariantTy>);

macro_rules! impl_str_eq {
    ($lhs:ty, $rhs: ty) => {
        #[allow(clippy::redundant_slicing)]
        impl<'a, 'b> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                self.to_str().eq(&other[..])
            }
        }

        impl<'a, 'b> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                self[..].eq(other.to_str())
            }
        }
    };
}

impl_str_eq!(VariantTy, str);
impl_str_eq!(VariantTy, &'a str);
impl_str_eq!(&'a VariantTy, str);
impl_str_eq!(VariantTy, String);
impl_str_eq!(&'a VariantTy, String);
impl_str_eq!(VariantType, str);
impl_str_eq!(VariantType, &'a str);
impl_str_eq!(VariantType, String);

impl Eq for VariantType {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ToValue;

    unsafe fn equal<T, U>(ptr1: *const T, ptr2: *const U) -> bool {
        from_glib(ffi::g_variant_type_equal(
            ptr1 as *const _,
            ptr2 as *const _,
        ))
    }

    #[test]
    fn new() {
        let ty = VariantTy::new("((iii)s)").unwrap();
        unsafe {
            assert!(equal(ty.as_ptr(), b"((iii)s)\0" as *const u8));
        }
    }

    #[test]
    fn new_empty() {
        assert!(VariantTy::new("").is_err());
    }

    #[test]
    fn new_with_nul() {
        assert!(VariantTy::new("((iii\0)s)").is_err());
    }

    #[test]
    fn new_too_short() {
        assert!(VariantTy::new("((iii").is_err());
    }

    #[test]
    fn new_too_long() {
        assert!(VariantTy::new("(iii)s").is_err());
    }

    #[test]
    fn eq() {
        let ty1 = VariantTy::new("((iii)s)").unwrap();
        let ty2 = VariantTy::new("((iii)s)").unwrap();
        assert_eq!(ty1, ty2);
        assert_eq!(ty1, "((iii)s)");
        unsafe {
            assert!(equal(ty1.as_ptr(), ty2.as_ptr()));
        }
    }

    #[test]
    fn ne() {
        let ty1 = VariantTy::new("((iii)s)").unwrap();
        let ty2 = VariantTy::new("((iii)o)").unwrap();
        assert_ne!(ty1, ty2);
        assert_ne!(ty1, "((iii)o)");
        unsafe {
            assert!(!equal(ty1.as_ptr(), ty2.as_ptr()));
        }
    }

    #[test]
    fn from_bytes() {
        unsafe {
            let ty = VariantTy::from_ptr(b"((iii)s)" as *const u8 as *const _);
            assert_eq!(ty, "((iii)s)");
            assert!(equal(ty.as_ptr(), "((iii)s)".as_ptr()));
        }
    }

    #[test]
    fn to_owned() {
        let ty1 = VariantTy::new("((iii)s)").unwrap();
        let ty2 = ty1.to_owned();
        assert_eq!(ty1, ty2);
        assert_eq!(ty2, "((iii)s)");
        unsafe {
            assert!(equal(ty1.as_ptr(), ty2.as_ptr()));
        }
    }

    #[test]
    fn value() {
        let ty1 = VariantType::new("*").unwrap();
        let tyv = ty1.to_value();
        let ty2 = tyv.get::<VariantType>().unwrap();
        assert_eq!(ty1, ty2);

        let ty3 = VariantTy::new("*").unwrap();
        let tyv2 = ty3.to_value();
        let ty4 = tyv2.get::<VariantType>().unwrap();
        assert_eq!(ty3, ty4);

        let ty5 = VariantTy::ANY;
        let tyv3 = ty5.to_value();
        let ty6 = tyv3.get::<VariantType>().unwrap();
        assert_eq!(ty5, ty6);
    }

    #[test]
    fn type_() {
        assert_eq!(VariantTy::static_type(), VariantType::static_type())
    }
}
