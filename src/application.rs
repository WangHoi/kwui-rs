use kwui_sys::*;
use std::ffi::CString;

/// The kwui running environment.
pub struct Application {
    inner: *mut kwui_Application,
}

type Closure<'a> = Box<dyn FnOnce() + 'a>;

impl Application {
    /// Create the kwui Application.
    pub fn new() -> Self {
        let args = std::env::args()
            .into_iter()
            .map(|a| CString::new(a).unwrap())
            .collect::<Vec<_>>();
        let mut args = args
            .iter()
            .map(|s| s.as_ptr() as *mut std::os::raw::c_char)
            .collect::<Vec<_>>();
        let argc = args.len() as _;
        let inner = unsafe {
            #[cfg(debug_assertions)]
            kwui_Application_enableScriptReload(true);

            kwui_Application_new(argc, args.as_mut_ptr())
        };

        Self { inner }
    }
    /// Check running in main thread
    pub fn is_main_thread() -> bool {
        unsafe { kwui_Application_isMainThread() }
    }
    /// Post a task to run in main thread,
    ///
    /// NOTE: if already on main thread, run the task immediately.
    pub fn run_in_main_thread<F: FnOnce()>(f: F) {
        let closure: Box<Closure> = Box::new(Box::new(f) as Closure);

        unsafe {
            kwui_Application_runInMainThread(Some(invoke_closure), Box::into_raw(closure) as _)
        }
    }
    /// Set resource directory to local folder
    pub fn set_resource_root_dir(&self, dir: &str) {
        let dir = CString::new(dir).unwrap();
        unsafe { kwui_Application_setResourceRootDir(self.inner, dir.as_ptr()) }
    }
    /// Load resource from packed data
    ///
    /// Users can pack resources with kwui-cli: `kwui pack-archive --help`
    pub fn set_resource_root_data(&self, data: &'static [u8]) {
        unsafe { kwui_Application_setResourceRootData(self.inner, data.as_ptr(), data.len()) }
    }
    /// Run the main app message loop until the application quits.
    pub fn exec(&self) -> i32 {
        unsafe { kwui_Application_exec(self.inner) }
    }
    /// Post quit app message
    pub fn quit() {
        unsafe { kwui_Application_quit() }
    }
}

unsafe extern "C" fn invoke_closure(udata: *mut ::std::os::raw::c_void) {
    eprintln!("invoke_closure called");
    let closure = std::ptr::read(udata as *mut Closure);
    closure();
}

impl Drop for Application {
    fn drop(&mut self) {
        unsafe { kwui_Application_delete(self.inner) }
    }
}
