use kwui_sys::*;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_int, c_void};

/// Wraps JavaScript value
pub struct ScriptValue {
    inner: *mut kwui_ScriptValue,
}
unsafe impl Send for ScriptValue {}
unsafe impl Sync for ScriptValue {}

impl ScriptValue {
    /// Make an explicit JavaScript value from `IntoScriptValue` trait
    pub fn from(v: impl IntoScriptValue) -> Self {
        match v.into_script_value() {
            Ok(v) => v,
            Err(_) => Self::default(),
        }
    }
    /// Make an explicit JavaScript `null` value
    pub fn new_null() -> Self {
        let inner = unsafe { kwui_ScriptValue_newNull() };
        Self { inner }
    }
    /// Make an explicit JavaScript `boolean` value
    pub fn new_bool(v: bool) -> Self {
        let inner = unsafe { kwui_ScriptValue_newBool(v) };
        Self { inner }
    }
    /// Make an explicit JavaScript `integer` value
    pub fn new_int(v: i32) -> Self {
        let inner = unsafe { kwui_ScriptValue_newInt(v) };
        Self { inner }
    }
    /// Make an explicit JavaScript `float` value
    pub fn new_double(v: f64) -> Self {
        let inner = unsafe { kwui_ScriptValue_newDouble(v) };
        Self { inner }
    }
    /// Make an explicit JavaScript `string` value
    pub fn new_string(v: &str) -> Self {
        let inner = unsafe { kwui_ScriptValue_newString(v.as_ptr() as _, v.len()) };
        Self { inner }
    }
    /// Make an explicit JavaScript `array` value
    pub fn new_array() -> Self {
        let inner = unsafe { kwui_ScriptValue_newArray() };
        Self { inner }
    }
    /// Make an explicit JavaScript `object` value
    pub fn new_object() -> Self {
        let inner = unsafe { kwui_ScriptValue_newObject() };
        Self { inner }
    }
    /// Retrieve value of sub-element at `idx`.
    pub fn get_value_by_index(&self, idx: usize) -> ScriptValue {
        let inner = unsafe { kwui_ScriptValue_get_by_index(self.inner, idx as _) };
        Self::from_inner(inner)
    }
    /// Retrieve value of sub-element at `idx`, then convert to `T` type.
    pub fn get_by_index<T: FromScriptValue + Default>(&self, idx: usize) -> T {
        T::from_script_value(&self.get_value_by_index(idx)).unwrap_or(T::default())
    }
    /// Try to retrieve value of sub-element at `idx`, then convert to `T` type.
    pub fn try_get_by_index<T: FromScriptValue>(&self, idx: usize) -> Result<T, ()> {
        T::from_script_value(&self.get_value_by_index(idx))
    }
    /// Insert or set value of the sub-element by `idx`.
    pub fn set_value_by_index(&self, idx: usize, v: ScriptValue) {
        unsafe { kwui_ScriptValue_set_by_index(self.inner, idx as _, v.inner) };
    }
    /// Insert or set value of the sub-element by `idx`.
    pub fn set_by_index(&mut self, idx: usize, v: impl IntoScriptValue) {
        let v = ScriptValue::from(v);
        unsafe { kwui_ScriptValue_set_by_index(self.inner, idx as _, v.inner) };
        std::mem::forget(v);
    }
    /// Retrieve value of sub-element at `key`.
    pub fn get_value_by_str(&self, key: &str) -> ScriptValue {
        let key = CString::new(key).unwrap_or_default();
        let inner = unsafe { kwui_ScriptValue_get_by_str(self.inner, key.as_ptr()) };
        Self::from_inner(inner)
    }
    /// Retrieve value of sub-element at `key`, then convert to `T` type.
    pub fn get_by_str<T: FromScriptValue + Default>(&self, key: &str) -> T {
        T::from_script_value(&self.get_value_by_str(key)).unwrap_or_default()
    }
    /// Try to retrieve value of sub-element at `idx`, then convert to `T` type.
    pub fn try_get_by_str<T: FromScriptValue>(&self, key: &str) -> Result<T, ()> {
        T::from_script_value(&self.get_value_by_str(key))
    }
    /// Insert or set value of the sub-element by `key`.
    pub fn set_by_str(&mut self, key: &str, v: impl IntoScriptValue) {
        let key = CString::new(key).unwrap_or_default();
        let v = ScriptValue::from(v);
        unsafe { kwui_ScriptValue_set_by_str(self.inner, key.as_ptr(), v.inner) };
        std::mem::forget(v);
    }
    /// Insert or set value of the sub-element by `key`.
    pub fn set_value_by_str(&mut self, key: &str, v: ScriptValue) {
        let key = CString::new(key).unwrap_or_default();
        unsafe { kwui_ScriptValue_set_by_str(self.inner, key.as_ptr(), v.inner) };
    }
    pub fn is_null(&self) -> bool {
        unsafe { kwui_ScriptValue_is_null(self.inner) }
    }
    pub fn is_bool(&self) -> bool {
        unsafe { kwui_ScriptValue_is_bool(self.inner) }
    }
    pub fn is_number(&self) -> bool {
        unsafe { kwui_ScriptValue_is_number(self.inner) }
    }
    pub fn is_string(&self) -> bool {
        unsafe { kwui_ScriptValue_is_string(self.inner) }
    }
    pub fn is_array(&self) -> bool {
        unsafe { kwui_ScriptValue_is_array(self.inner) }
    }
    pub fn is_object(&self) -> bool {
        unsafe { kwui_ScriptValue_is_object(self.inner) }
    }
    /// Value to boolean.
    pub fn to_bool(&self) -> bool {
        unsafe { kwui_ScriptValue_to_bool(self.inner) }
    }
    /// Value to double.
    pub fn to_double(&self) -> f64 {
        unsafe { kwui_ScriptValue_to_double(self.inner) }
    }
    /// Value to integer.
    pub fn to_int(&self) -> i32 {
        unsafe { kwui_ScriptValue_to_int(self.inner) }
    }
    /// Value to string.
    pub fn to_string(&self) -> String {
        let buf = unsafe {
            let mut len = 0;
            let buf = kwui_ScriptValue_to_string(self.inner, &mut len);
            std::slice::from_raw_parts(buf as *const u8, len)
        };
        String::from_utf8_lossy(buf).to_string()
    }
    /// Length of array or object value.
    pub fn length(&self) -> usize {
        unsafe { kwui_ScriptValue_length(self.inner) }
    }
    /// Visiting all values of an array.
    pub fn visit_array<F: FnMut(usize, &ScriptValue)>(&self, callback: F) {
        //let callback = Box::into_raw(Box::new(callback)) as *mut c_void;
        unsafe extern "C" fn visit_array_callback<F: FnMut(usize, &ScriptValue)>(
            index: c_int,
            val: *const kwui_ScriptValue,
            udata: *mut c_void,
        ) {
            let val = ScriptValue::from_inner(val as _);
            (*(udata as *mut F))(index as _, &val);
            std::mem::forget(val);
        }
        unsafe {
            kwui_ScriptValue_visitArray(
                self.inner,
                Some(visit_array_callback::<F>),
                &callback as *const F as _,
            );
        }
    }
    /// Visiting all values of an object, in arbitrary order.
    pub fn visit_object<F: FnMut(&str, &ScriptValue)>(&self, callback: F) {
        //let callback = Box::into_raw(Box::new(callback)) as *mut c_void;
        unsafe extern "C" fn visit_object_callback<F: FnMut(&str, &ScriptValue)>(
            key: *const std::os::raw::c_char,
            key_len: usize,
            val: *const kwui_ScriptValue,
            udata: *mut c_void,
        ) {
            let key = std::slice::from_raw_parts(key as *const u8, key_len);
            let val = ScriptValue::from_inner(val as _);
            (*(udata as *mut F))(std::str::from_utf8_unchecked(key), &val);
            std::mem::forget(val);
        }
        unsafe {
            kwui_ScriptValue_visitObject(
                self.inner,
                Some(visit_object_callback::<F>),
                &callback as *const F as _,
            );
        }
    }

    pub(crate) fn from_inner(inner: *mut kwui_ScriptValue) -> Self {
        // eprintln!("from_inner {:?}", inner);
        Self { inner }
    }
    pub(crate) fn inner(&self) -> *mut kwui_ScriptValue {
        self.inner
    }
    pub(crate) fn leak(self) -> *mut kwui_ScriptValue {
        // eprintln!("leak {:?}", self.inner);
        let inner = self.inner;
        std::mem::forget(self);
        inner
    }
}

impl Drop for ScriptValue {
    fn drop(&mut self) {
        // eprintln!("drop {:?}", self.inner);
        unsafe {
            kwui_ScriptValue_delete(self.inner);
        }
    }
}

impl std::fmt::Debug for ScriptValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            f.write_str("ScriptValue::Null")
        } else if self.is_bool() {
            f.write_fmt(format_args!("ScriptValue::Bool({})", self.to_bool()))
        } else if self.is_number() {
            f.write_fmt(format_args!("ScriptValue::Number({:?})", self.to_double()))
        } else if self.is_string() {
            f.write_fmt(format_args!("ScriptValue::String({:?})", self.to_string()))
        } else if self.is_array() {
            f.write_str("ScriptValue::Array")?;
            let mut dl = f.debug_list();
            self.visit_array(|index, value| {
                let _ = dl.entry(value);
            });
            dl.finish()
        } else if self.is_object() {
            let mut ds = f.debug_struct("ScriptValue::Object");
            self.visit_object(|key, value| {
                let _ = ds.field(key, value);
            });
            ds.finish()
        } else {
            f.write_str("ScriptValue::UnknownType")
        }
    }
}

/// Convert `Script` to rust type
pub trait FromScriptValue: Sized {
    fn from_script_value(value: &ScriptValue) -> Result<Self, ()>;
}

/// Convert rust type to `ScriptValue`
pub trait IntoScriptValue {
    fn into_script_value(self) -> Result<ScriptValue, ()>;
}

impl Default for ScriptValue {
    fn default() -> Self {
        let inner = unsafe { kwui_ScriptValue_newNull() };
        Self { inner }
    }
}
impl FromScriptValue for () {
    fn from_script_value(value: &ScriptValue) -> Result<Self, ()> {
        Ok(())
    }
}

impl IntoScriptValue for () {
    fn into_script_value(self) -> Result<ScriptValue, ()> {
        let inner = unsafe { kwui_ScriptValue_newNull() };
        Ok(ScriptValue::from_inner(inner))
    }
}
impl FromScriptValue for bool {
    fn from_script_value(value: &ScriptValue) -> Result<Self, ()> {
        unsafe { Ok(kwui_ScriptValue_to_bool(value.inner)) }
    }
}

impl IntoScriptValue for bool {
    fn into_script_value(self) -> Result<ScriptValue, ()> {
        let inner = unsafe { kwui_ScriptValue_newBool(self) };
        Ok(ScriptValue::from_inner(inner))
    }
}

macro_rules! number_from_script_value {
    ($ty:ident) => {
        impl FromScriptValue for $ty {
            fn from_script_value(value: &ScriptValue) -> Result<Self, ()> {
                unsafe { Ok(kwui_ScriptValue_to_double(value.inner) as _) }
            }
        }
    };
}
macro_rules! number_into_script_value {
    ($ty:ident) => {
        impl IntoScriptValue for $ty {
            fn into_script_value(self) -> Result<ScriptValue, ()> {
                let inner = unsafe { kwui_ScriptValue_newDouble(self as _) };
                Ok(ScriptValue::from_inner(inner))
            }
        }
    };
}
number_from_script_value!(i8);
number_from_script_value!(u8);
number_from_script_value!(i16);
number_from_script_value!(u16);
number_from_script_value!(i32);
number_from_script_value!(u32);
number_from_script_value!(i64);
number_from_script_value!(u64);
number_from_script_value!(isize);
number_from_script_value!(usize);
number_from_script_value!(f32);
number_from_script_value!(f64);

number_into_script_value!(i8);
number_into_script_value!(u8);
number_into_script_value!(i16);
number_into_script_value!(u16);
number_into_script_value!(i32);
number_into_script_value!(u32);
number_into_script_value!(i64);
number_into_script_value!(u64);
number_into_script_value!(isize);
number_into_script_value!(usize);
number_into_script_value!(f32);
number_into_script_value!(f64);

impl FromScriptValue for String {
    fn from_script_value(value: &ScriptValue) -> Result<Self, ()> {
        unsafe {
            let mut len = 0;
            let data = kwui_ScriptValue_to_string(value.inner, &mut len);
            let slice = std::slice::from_raw_parts(data as *const u8, len);
            Ok(String::from_utf8_lossy(slice).to_string())
        }
    }
}

impl IntoScriptValue for String {
    fn into_script_value(self) -> Result<ScriptValue, ()> {
        let inner = unsafe { kwui_ScriptValue_newString(self.as_ptr() as _, self.len()) };
        Ok(ScriptValue::from_inner(inner))
    }
}

impl IntoScriptValue for &str {
    fn into_script_value(self) -> Result<ScriptValue, ()> {
        let inner = unsafe { kwui_ScriptValue_newString(self.as_ptr() as _, self.len()) };
        Ok(ScriptValue::from_inner(inner))
    }
}

impl<K: From<String> + Eq + std::hash::Hash, V: FromScriptValue + Default> FromScriptValue
    for HashMap<K, V>
{
    fn from_script_value(value: &ScriptValue) -> Result<Self, ()> {
        let mut obj = Self::new();
        value.visit_object(|k, v| {
            let k = k.to_string().into();
            let v = V::from_script_value(v).unwrap_or_default();
            obj.insert(k, v);
        });
        Ok(obj)
    }
}

impl<K: AsRef<str>, V: IntoScriptValue> IntoScriptValue for HashMap<K, V> {
    fn into_script_value(self) -> Result<ScriptValue, ()> {
        let mut obj = ScriptValue::new_object();
        for (k, v) in self.into_iter() {
            obj.set_value_by_str(k.as_ref(), v.into_script_value()?);
        }
        Ok(obj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array() {
        let mut aa = ScriptValue::new_array();
        aa.set_by_index(0, 1);
        aa.set_by_index(1, 2);
        aa.set_by_index(2, 3);
        assert_eq!(aa.length(), 3);
        assert_eq!(aa.get_by_index::<usize>(0), 1);

        aa.visit_array(|index, val| {
            assert_eq!(index as i32 + 1, val.to_int());
        });
    }
}
