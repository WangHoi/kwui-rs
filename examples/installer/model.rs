use std::{cell::RefCell, rc::Rc, sync::Arc};

use kwui::{IntoScriptValue, ScriptEngine, ScriptEventHandler, ScriptValue};

struct Product {
    display_name: String,
    version: String,
}

impl IntoScriptValue for Product {
    fn into_script_value(self) -> Result<ScriptValue, ()> {
        let mut obj = ScriptValue::new_object();
        obj.set_by_str("displayName", self.display_name);
        obj.set_by_str("version", self.version);
        Ok(obj)
    }
}

struct TargetDir {
    dir: String,
    valid: bool,
}

impl IntoScriptValue for TargetDir {
    fn into_script_value(self) -> Result<ScriptValue, ()> {
        let mut obj = ScriptValue::new_array();
        obj.set_by_index(0, self.dir);
        obj.set_by_index(1, self.valid);
        Ok(obj)
    }
}

struct FreeSpace {
    estimated_size_mb: usize,
    target_free_space_gb: Option<usize>,
}

impl IntoScriptValue for FreeSpace {
    fn into_script_value(self) -> Result<ScriptValue, ()> {
        let mut obj = ScriptValue::new_array();
        obj.set_by_index(0, self.estimated_size_mb);
        if let Some(space) = self.target_free_space_gb {
            obj.set_by_index(1, space);
        }
        Ok(obj)
    }
}

pub struct Model {
    inner: Rc<RefCell<ModelState>>,
}

struct ModelState {
    dialog_id: String,
    main_page_expanded: bool,
    expand_button_clicked_handler: Option<ScriptEventHandler>,
}

impl ModelState {
    fn new() -> Self {
        Self {
            dialog_id: String::new(),
            main_page_expanded: false,
            expand_button_clicked_handler: None,
        }
    }
    fn get_current_page(&self) -> usize {
        0
    }
    fn get_target_dir(&self) -> TargetDir {
        TargetDir {
            dir: "C:\\".into(),
            valid: true,
        }
    }
    fn get_free_space(&self) -> FreeSpace {
        FreeSpace {
            estimated_size_mb: 123,
            target_free_space_gb: 234.into(),
        }
    }
    fn is_main_page_expanded(&self) -> bool {
        self.main_page_expanded
    }
    fn on_expand_button_clicked(&mut self) {
        self.main_page_expanded = !self.main_page_expanded;
        eprintln!("on_expand_button_clicked {}", self.main_page_expanded);
        ScriptEngine::post_event0("install-dialog:main-page-expanded");
    }
    fn get_install_progress(&self) -> f32 {
        30.0
    }
}
impl Model {
    pub fn new() -> Self {
        let inner = Rc::new(RefCell::new(ModelState::new()));
        {
            let m = Rc::clone(&inner);
            ScriptEngine::add_global_function("getCurrentPage", move |_: ()| -> usize {
                m.borrow().get_current_page()
            });
        }
        {
            let m = Rc::clone(&inner);
            ScriptEngine::add_global_function("getTargetDir", move |_: ()| -> TargetDir {
                m.borrow().get_target_dir()
            });
        }
        {
            let m = Rc::clone(&inner);
            ScriptEngine::add_global_function("getFreeSpace", move |_: ()| -> FreeSpace {
                m.borrow().get_free_space()
            });
        }
        {
            let m = Rc::clone(&inner);
            ScriptEngine::add_global_function("isMainPageExpanded", move |_: ()| -> bool {
                m.borrow().is_main_page_expanded()
            });
        }
        {
            let m = Rc::clone(&inner);
            ScriptEngine::add_global_function("getInstallProgress", move |_: ()| -> f32 {
                m.borrow().get_install_progress()
            });
        }
        {
            let m = Rc::clone(&inner);
            inner.borrow_mut().expand_button_clicked_handler = ScriptEngine::add_event_listener(
                "install-dialog:expand-button-clicked",
                move |_: ()| m.borrow_mut().on_expand_button_clicked(),
            )
            .into();
        }
        Self { inner }
    }

    pub fn start_install(&self) {
        let arg = Product {
            display_name: "测试产品".into(),
            version: "1.0.1".into(),
        }
        .into_script_value()
        .unwrap();
        let dialog_id = ScriptEngine::call_global_function("showInstallDialog", &[arg]).to_string();
        self.inner.borrow_mut().dialog_id = dialog_id;
    }
}

impl Drop for Model {
    fn drop(&mut self) {}
}
