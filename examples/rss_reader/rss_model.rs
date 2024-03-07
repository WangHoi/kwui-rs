use kwui::{IntoScriptValue, ScriptEngine, ScriptEventHandler, ScriptValue};
use reqwest;
use rss;
use std::cell::RefCell;
use tokio;

const FEED_URL: &str = "https://feed.williamlong.info/";

#[derive(Debug, Clone, Default)]
struct Channel {
    title: String,
    items: Vec<ChannelItem>,
}

#[derive(Debug, Clone, Default)]
struct ChannelItem {
    title: String,
    description: String,
}

impl IntoScriptValue for Channel {
    fn into_script_value(self) -> Result<ScriptValue, ()> {
        let mut obj = ScriptValue::new_object();
        obj.set_by_str("title", self.title);
        let mut items = ScriptValue::new_array();
        for (i, v) in self.items.into_iter().enumerate() {
            items.set_by_index(i, v);
        }
        obj.set_value_by_str("items", items);
        Ok(obj)
    }
}

impl IntoScriptValue for ChannelItem {
    fn into_script_value(self) -> Result<ScriptValue, ()> {
        let mut obj = ScriptValue::new_object();
        obj.set_by_str("title", self.title);
        obj.set_by_str("description", self.description);
        Ok(obj)
    }
}

pub struct Model;

#[derive(Default)]
struct ModelState {
    channel: Option<Channel>,
    on_channel_loaded_handler: Option<ScriptEventHandler>,
    rt: Option<tokio::runtime::Runtime>,
}

thread_local! {
    static MODEL: RefCell<ModelState> = RefCell::new(ModelState::new());
}

impl ModelState {
    fn new() -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        Self {
            channel: None,
            on_channel_loaded_handler: None,
            rt: rt.into(),
        }
    }
}

impl Model {
    pub fn init() {
        ScriptEngine::add_global_function("getChannel", Model::get_channel);
        ScriptEngine::add_global_function("reloadChannel", Model::reload_channel);
    }
    fn reload_channel(_: ()) {
        MODEL.with_borrow(|m| {
            m.rt.as_ref().unwrap().spawn(Model::do_load_channel());
        });
    }
    fn get_channel(_: ()) -> Channel {
        MODEL.with_borrow(|m| m.channel.clone().unwrap_or_default())
    }
    pub fn deinit() {
        MODEL.take();
    }

    fn on_channel_loaded(channel: Channel) {
        eprintln!("on_channel_loaded, main_thread {}", kwui::Application::is_main_thread());
        let ch = channel.clone();
        MODEL.with_borrow_mut(|m| m.channel.replace(ch));
        ScriptEngine::post_event1("main-dialog:channel-loaded", channel);
    }
    async fn do_load_channel() -> anyhow::Result<()> {
        eprintln!("load channel");
        let content = reqwest::get(FEED_URL)
            .await
            .map_err(|e| {
                eprintln!("get error: {}", e);
                e
            })?
            .bytes()
            .await
            .map_err(|e| {
                eprintln!("get body error: {}", e);
                e
            })?;
        let rss_chan = rss::Channel::read_from(&content[..]).map_err(|e| {
            eprintln!("parse channel error: {}", e);
            e
        })?;
        let chan = Channel {
            title: rss_chan.title().to_string(),
            items: rss_chan
                .items()
                .iter()
                .map(|item| ChannelItem {
                    title: item.title().unwrap_or_default().to_string(),
                    description: item.description().unwrap_or_default().to_string(),
                })
                .collect(),
        };
        eprintln!("loaded channel items.len={}", chan.items.len());
        kwui::Application::run_in_main_thread(move || {
            Model::on_channel_loaded(chan);
        });
        Ok(())
    }
}

impl Drop for Model {
    fn drop(&mut self) {}
}
