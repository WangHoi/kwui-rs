use kwui::{IntoScriptValue, ScriptEngine, ScriptEventHandler, ScriptValue};
use std::cell::RefCell;

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

pub struct Model;

struct ModelState {
    dialog_id: String,
    current_page: &'static str,
    main_page_expanded: bool,
    install_progress: f64,
    expand_button_clicked_handler: Option<ScriptEventHandler>,
    start_button_clicked_handler: Option<ScriptEventHandler>,
    done_button_clicked_handler: Option<ScriptEventHandler>,
    animation_event_handler: Option<ScriptEventHandler>,
}

thread_local! {
    static MODEL: RefCell<ModelState> = RefCell::new(ModelState::new());
}

impl ModelState {
    fn new() -> Self {
        Self {
            dialog_id: String::new(),
            current_page: "main",
            main_page_expanded: false,
            install_progress: 0.0f64,
            expand_button_clicked_handler: None,
            start_button_clicked_handler: None,
            done_button_clicked_handler: None,
            animation_event_handler: None,
        }
    }
}
impl Model {
    pub fn init() {
        ScriptEngine::add_global_function("getCurrentPage", Model::get_current_page);
        ScriptEngine::add_global_function("getTargetDir", Model::get_target_dir);
        ScriptEngine::add_global_function("getFreeSpace", Model::get_free_space);
        ScriptEngine::add_global_function("isMainPageExpanded", Model::is_main_page_expanded);
        ScriptEngine::add_global_function("getInstallProgress", Model::get_install_progress);

        MODEL.with_borrow_mut(|m| {
            m.expand_button_clicked_handler = ScriptEngine::add_event_listener(
                "install-dialog:expand-button-clicked",
                Model::on_expand_button_clicked,
            )
            .into();
        });
        MODEL.with_borrow_mut(|m| {
            m.start_button_clicked_handler = ScriptEngine::add_event_listener(
                "install-dialog:start-button-clicked",
                Model::on_start_button_clicked,
            )
            .into();
        });
        MODEL.with_borrow_mut(|m| {
            m.done_button_clicked_handler = ScriptEngine::add_event_listener(
                "install-dialog:done-button-clicked",
                Model::on_done_button_clicked,
            )
            .into();
        });
        MODEL.with_borrow_mut(|m| {
            m.animation_event_handler = ScriptEngine::add_event_listener(
                "dialog:animation-event",
                Model::on_animation_event,
            )
            .into();
        });
    }
    fn get_current_page(_: ()) -> &'static str {
        MODEL.with_borrow(|m| m.current_page)
    }
    fn get_target_dir(_: ()) -> TargetDir {
        TargetDir {
            dir: "C:\\".into(),
            valid: true,
        }
    }
    fn get_free_space(_: ()) -> FreeSpace {
        FreeSpace {
            estimated_size_mb: 123,
            target_free_space_gb: 234.into(),
        }
    }
    fn is_main_page_expanded(_: ()) -> bool {
        MODEL.with_borrow(|m| m.main_page_expanded)
    }
    fn get_install_progress(_: ()) -> f64 {
        MODEL.with_borrow(|m| m.install_progress)
    }

    fn on_expand_button_clicked(_: ()) {
        MODEL.with_borrow_mut(|m| m.main_page_expanded = !m.main_page_expanded);
        ScriptEngine::post_event0("install-dialog:main-page-expanded");
    }
    fn on_start_button_clicked(_: ()) {
        eprintln!("on_start_button_clicked");
        MODEL.with_borrow_mut(|m| m.current_page = "progress");
        ScriptEngine::post_event0("install-dialog:current-page-changed");
    }
    fn on_done_button_clicked(_: ()) {
        let dialog_id = MODEL.with_borrow(|m| m.dialog_id.clone()).into_script_value().unwrap();
        ScriptEngine::call_global_function("closeDialog", &[dialog_id]);
    }
    fn on_animation_event(_: ()) {
        let (new_progress, notify) = MODEL.with_borrow_mut(|m| -> (f64, bool) {
            if m.current_page == "progress" {
                if m.install_progress < 1.0 {
                    m.install_progress += 0.2 * 1.0 / 60.0;
                    if m.install_progress > 1.0 {
                        m.install_progress = 1.0;
                        return (m.install_progress, true);
                    } else {
                        return (m.install_progress, true);
                    }
                } else {
                    return (m.install_progress, false);
                }
            } else {
                return (m.install_progress, false);
            }
        });
        
        if notify {
            ScriptEngine::post_event1("install-dialog:progress-changed", new_progress);
            if new_progress >= 1.0 {
                MODEL.with_borrow_mut(|m| m.current_page = "done");
                ScriptEngine::post_event0("install-dialog:current-page-changed");
            }
        }
    }
    pub fn start_install() {
        let arg = Product {
            display_name: "测试产品".into(),
            version: "1.0.1".into(),
        }
        .into_script_value()
        .unwrap();
        let dialog_id = ScriptEngine::call_global_function("showInstallDialog", &[arg]).to_string();
        MODEL.with_borrow_mut(|m| m.dialog_id = dialog_id);
    }
}

impl Drop for Model {
    fn drop(&mut self) {}
}
