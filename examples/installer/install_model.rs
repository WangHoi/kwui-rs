use kwui::{Application, IntoScriptValue, ScriptEngine, ScriptEventHandler, ScriptValue};
use std::cell::RefCell;

struct Product {
    display_name: &'static str,
    version: &'static str,
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

#[derive(Default)]
struct ModelState {
    // Main dialog state
    dialog_id: String,
    current_page: &'static str,
    main_page_expanded: bool,
    install_progress: f64,
    expand_button_clicked_handler: Option<ScriptEventHandler>,
    start_button_clicked_handler: Option<ScriptEventHandler>,
    done_button_clicked_handler: Option<ScriptEventHandler>,
    animation_event_handler: Option<ScriptEventHandler>,
    request_close_handler: Option<ScriptEventHandler>,
    enter_key_down_handler: Option<ScriptEventHandler>,

    // Confirm dialog state
    confirm_dialog_id: String,
    confirm_dialog_action_button_clicked_handler: Option<ScriptEventHandler>,
    confirm_dialog_cancel_button_clicked_handler: Option<ScriptEventHandler>,
}

thread_local! {
    static MODEL: RefCell<ModelState> = RefCell::new(ModelState::new());
}

const PRODUCT: Product = Product {
    display_name: "测试产品",
    version: "1.0.1",
};

impl ModelState {
    fn new() -> Self {
        Self {
            current_page: "main",
            ..Default::default()
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
            m.start_button_clicked_handler = ScriptEngine::add_event_listener(
                "install-dialog:start-button-clicked",
                Model::on_start_button_clicked,
            )
            .into();
            m.done_button_clicked_handler = ScriptEngine::add_event_listener(
                "install-dialog:done-button-clicked",
                Model::on_done_button_clicked,
            )
            .into();
            m.animation_event_handler = ScriptEngine::add_event_listener(
                "dialog:animation-event",
                Model::on_animation_event,
            )
            .into();
            m.request_close_handler =
                ScriptEngine::add_event_listener("dialog:request-close", Model::on_request_close)
                    .into();
            m.enter_key_down_handler =
                ScriptEngine::add_event_listener("dialog:enter-key-down", Model::on_enter_key_down)
                    .into();
            m.confirm_dialog_action_button_clicked_handler = ScriptEngine::add_event_listener(
                "confirm-dialog:action-button-clicked",
                Model::on_confirm_dialog_action_button_clicked,
            )
            .into();
            m.confirm_dialog_cancel_button_clicked_handler = ScriptEngine::add_event_listener(
                "confirm-dialog:cancel-button-clicked",
                Model::on_confirm_dialog_cancel_button_clicked,
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
        Model::close_main_dialog();
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
    fn on_request_close((_event, did): (String, String)) -> bool {
        let (is_me, current_page) = MODEL.with_borrow(|m| (m.dialog_id == did, m.current_page));
        // eprintln!("on_request_close did={} is_me={}", did, is_me);
        if !is_me {
            return false;
        }

        if current_page != "done" {
            let mut arg = ScriptValue::new_object();
            arg.set_by_str("title", format!("{}安装向导", PRODUCT.display_name));
            arg.set_by_str("label", format!("确定要停止{}安装？", PRODUCT.display_name));
            arg.set_by_str("action_btn", "继续安装");
            arg.set_by_str("cancel_btn", "停止");
            let dialog_id = ScriptEngine::call_global_function("showConfirmDialog", &[arg]);
            MODEL.with_borrow_mut(|m| m.confirm_dialog_id = dialog_id.to_string());

            // return true to cancel dialog close
            true
        } else {
            false
        }
    }
    fn on_enter_key_down((_event, did): ((), String)) {
        let (is_me, current_page) = MODEL.with_borrow(|m| (m.dialog_id == did, m.current_page));
        if !is_me {
            return;
        }
        if current_page == "main" {
            Model::on_start_button_clicked(());
        } else if current_page == "done" {
            Model::on_done_button_clicked(());
        }
    }

    fn on_confirm_dialog_action_button_clicked(_: ()) {
        Model::close_confirm_dialog();
    }
    fn on_confirm_dialog_cancel_button_clicked(_: ()) {
        Model::close_confirm_dialog();
        Model::close_main_dialog();        
    }
    fn close_confirm_dialog() {
        let dialog_id: String = MODEL.with_borrow_mut(|m| {
            let did = m.confirm_dialog_id.clone();
            m.confirm_dialog_id.clear();
            return did;
        });
        let arg = dialog_id.into_script_value().unwrap();
        ScriptEngine::call_global_function("closeDialog", &[arg]);
    }
    fn close_main_dialog() {
        let dialog_id: String = MODEL.with_borrow_mut(|m| {
            let did = m.dialog_id.clone();
            m.dialog_id.clear();
            return did;
        });
        let arg = dialog_id.into_script_value().unwrap();
        ScriptEngine::call_global_function("closeDialog", &[arg]);
    }
    pub fn start_install() {
        let arg = PRODUCT.into_script_value().unwrap();
        let dialog_id = ScriptEngine::call_global_function("showInstallDialog", &[arg]).to_string();
        MODEL.with_borrow_mut(|m| m.dialog_id = dialog_id.to_string());
    }
    pub fn deinit() {
        MODEL.take();
    }
}

impl Drop for Model {
    fn drop(&mut self) {}
}
