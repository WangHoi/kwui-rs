use kwui_sys::*;
use log;
use std::{ffi::CString, io::Read};

use crate::script_value::{FromScriptValue, IntoScriptValue, ScriptValue};

/// The global script engine
///
/// For rust and JavaScript interop.
pub struct ScriptEngine;

/// Handler retrieved from `add_event_handler`, for removing event handler later.
pub struct ScriptEventHandler {
    event: CString,
    inner: *mut std::os::raw::c_void,
}

impl Drop for ScriptEventHandler {
    fn drop(&mut self) {
        ScriptEngine::remove_event_listener(self);
    }
}

impl ScriptEngine {
    /// Load and run script file
    ///
    /// To load from resource, use ":/" prefix, Example: load_file(":/entry.js")
    pub fn load_file(path: &str) {
        let path = CString::new(path).unwrap();
        unsafe { kwui_ScriptEngine_loadFile(path.as_ptr()) }
    }
    /// Call JavaScript function
    ///
    /// See also: `make_args`, `ScriptValue`
    pub fn call_global_function(name: &str, args: &[ScriptValue]) -> ScriptValue {
        let name = CString::new(name).unwrap();
        let mut args = args
            .iter()
            .map(|s| s.inner())
            .collect::<Box<[*mut kwui_ScriptValue]>>();
        let inner = unsafe {
            kwui_ScriptEngine_callGlobalFunction(name.as_ptr(), args.len() as _, args.as_mut_ptr())
        };
        ScriptValue::from_inner(inner)
    }
    /// Export Rust function to JavaScript
    pub fn add_global_function<R, Fun, Args>(name: &str, func: Fun)
    where
        Fun: ScriptFunction<R, Args> + 'static,
    {
        let name = CString::new(name).unwrap();
        let closure: Box<Callback> = Box::new(Box::new(move |params| -> Result<ScriptValue, ()> {
            func.invoke(params)
        }) as Callback);
        unsafe {
            kwui_ScriptEngine_addGlobalFunction(
                name.as_ptr(),
                Some(invoke_closure),
                Box::into_raw(closure) as _,
            );
        }
    }
    /// Undo export of Rust function
    pub fn remove_global_function(name: &str) {
        let name = CString::new(name).unwrap();
        unsafe {
            kwui_ScriptEngine_removeGlobalFunction(name.as_ptr());
        }
    }
    /// Add an event listener callback Rust function
    pub fn add_event_listener<R, Fun, Args>(event: &str, func: Fun) -> ScriptEventHandler
    where
        R: IntoScriptValue,
        Fun: ScriptFunction<R, Args> + 'static,
    {
        let c_event = CString::new(event).unwrap();
        let e = event.to_string();
        let closure: Box<Callback> = Box::new(Box::new(
            move |args: &[ScriptValue]| -> Result<ScriptValue, ()> {
                func.invoke(args)
            },
        ) as Callback);
        let inner = Box::into_raw(closure) as _;
        unsafe {
            kwui_ScriptEngine_addEventListener(c_event.as_ptr(), Some(invoke_closure), inner);
        }
        ScriptEventHandler {
            event: c_event,
            inner,
        }
    }
    /// Remove an event listener callback Rust function
    pub fn remove_event_listener(handler: &mut ScriptEventHandler) {
        // eprintln!("remove_event_listener {}", handler.event.to_string_lossy());
        unsafe {
            kwui_ScriptEngine_removeEventListener(
                handler.event.as_ptr(),
                Some(invoke_closure),
                handler.inner,
            );
        }
    }
    /// Trigger an event, both JavaScript and Rust event listeners will be notified.
    pub fn post_event0(event: &str) {
        let c_event = CString::new(event).unwrap();
        unsafe {
            kwui_ScriptEngine_postEvent0(c_event.as_ptr());
        }
    }
    /// Trigger an event with data, both JavaScript and Rust event listeners will be notified.
    pub fn post_event1(event: &str, data: impl IntoScriptValue) {
        let c_event = CString::new(event).unwrap();
        let data = data.into_script_value();
        if let Ok(data) = data {
            unsafe {
                kwui_ScriptEngine_postEvent1(c_event.as_ptr(), data.inner());
            }
        } else {
            log::warn!("ScriptEngine::post_event '{}' failed", event);
        }
    }
}

/// Make script function args, return `[ScriptValue]`
#[macro_export]
macro_rules! make_args {
	() => { { let args : [$crate::value::Value; 0] = []; args } };

	( $($s:expr),* ) => {
		{
			let args = [
			$(
				$crate::script_value::ScriptValue::from($s)
			 ),*
			];
			args
		}
	};
}

type Callback<'a> = Box<dyn Fn(&'a [ScriptValue]) -> Result<ScriptValue, ()> + 'a>;

pub(crate) trait ScriptFuntionParams: Sized {
    fn from_params(params: &[ScriptValue]) -> Result<Self, ()>;
}

macro_rules! impl_script_function_param {
    () => {
        impl_script_function_param!(@expand 0usize ; ;);
    };
    ($($p:ident),*) => {
        impl_script_function_param!(@expand 0usize ; ; $($p)*);
    };
    (@expand $n:expr ; $([$idx:expr, $pa:ident]),* ; $a:ident $($p:ident)*) => {
        impl_script_function_param!(@expand $n + 1usize; $([$idx, $pa],)* [$n, $a] ; $($p)*);
    };
    (@expand $n:expr ; $([$idx:expr, $pa:tt]),* ;) => {
        impl<$($pa,)*> ScriptFuntionParams for ($($pa,)*)
        where
        $(
            $pa: FromScriptValue + Default,
        )*
        {
            fn from_params(param: &[ScriptValue]) -> Result<Self, ()> {
                Ok((
                    $(
                        param
                            .get($idx)
                            .and_then(|arg| $pa::from_script_value(arg).ok())
                            .unwrap_or_default(),
                    )*
                ))
            }
        }
    };
}
impl_script_function_param!();
impl_script_function_param!(A1);
impl_script_function_param!(A1, A2);
impl_script_function_param!(A1, A2, A3);
impl_script_function_param!(A1, A2, A3, A4);
impl_script_function_param!(A1, A2, A3, A4, A5);
impl_script_function_param!(A1, A2, A3, A4, A5, A6);
impl_script_function_param!(A1, A2, A3, A4, A5, A6, A7);
impl_script_function_param!(A1, A2, A3, A4, A5, A6, A7, A8);

/// An invokable function with specified signature.
pub trait ScriptFunction<R, Args> {
    fn invoke(&self, params: &[ScriptValue]) -> Result<ScriptValue, ()>;
}
macro_rules! impl_script_function {
    ($($arg:ident),*) => {
        #[allow(non_snake_case)]
        impl<Fun, R, $($arg,)*> ScriptFunction<R, ($($arg,)*)> for Fun
        where
            R: IntoScriptValue,
            ($($arg,)*): ScriptFuntionParams,
            Fun: Fn($($arg),*) -> R,
        {
            fn invoke(&self, params: &[ScriptValue]) -> Result<ScriptValue, ()> {
                let ($($arg,)*) = ScriptFuntionParams::from_params(params)?;
                self($($arg,)*).into_script_value()
            }
        }
    };
}
impl_script_function!();
impl_script_function!(A1);
impl_script_function!(A1, A2);
impl_script_function!(A1, A2, A3);
impl_script_function!(A1, A2, A3, A4);
impl_script_function!(A1, A2, A3, A4, A5);
impl_script_function!(A1, A2, A3, A4, A5, A6);
impl_script_function!(A1, A2, A3, A4, A5, A6, A7);
impl_script_function!(A1, A2, A3, A4, A5, A6, A7, A8);

unsafe extern "C" fn invoke_closure(
    argc: ::std::os::raw::c_int,
    argv: *mut *mut kwui_ScriptValue,
    udata: *mut ::std::os::raw::c_void,
) -> *mut kwui_ScriptValue {
    // eprintln!("invoke_closure called argc={} argv={:?}", argc, argv);
    let closure = (udata as *mut Callback).as_mut().unwrap();
    let args: Vec<ScriptValue> = if argc > 0 {
        std::slice::from_raw_parts(argv, argc as _)
            .iter()
            .map(|inner| ScriptValue::from_inner(*inner))
            .collect()
    } else {
        Vec::new()
    };
    let ret = closure(&args);
    for a in args.into_iter() {
        a.leak();
    }
    match ret {
        Ok(ret) => ret.leak(),
        Err(_) => kwui_ScriptValue_newNull(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Application;

    fn f0() {
        eprintln!("f0 called");
    }

    fn f1(arg0: String, arg1:f32) {
        eprintln!("f1 called with {:?} {:?}", arg0, arg1);
    }

    fn on_test_event(evt: String, arg: f32) {
        eprintln!("on_test_event evt={}, arg={}", evt, arg);
    }

    #[test]
    fn global_func() {
        let app = Application::new();
        ScriptEngine::add_global_function("f0", f0);
        ScriptEngine::call_global_function("f0", &[]);

        ScriptEngine::add_global_function("f1", f1);
        ScriptEngine::call_global_function("f1", &make_args!("a", 2));

        let a = on_test_event as *const ();
        let _handler = ScriptEngine::add_event_listener("test-event", on_test_event);
        ScriptEngine::post_event1("test-event", 1.23);
    }
}
