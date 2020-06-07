use std::path::Path;
use std::thread;

use greenhorn::components::checkbox;
use greenhorn::dialog::{FileFilter, FileOpenDialog, FileOpenMsg, FileSaveDialog, FileSaveMsg};
use greenhorn::prelude::*;

use merge_tool::config::{Config, FwConfig};

use crate::fw_config::{FwPane, FwMsg};
use crate::text_field::{TextFieldMsg, TextField};
use arrayvec::ArrayVec;
use chrono::{Local, Timelike};

#[derive(Debug)]
pub enum Msg {
    Open,
    SaveAs,
    ConfigSavedAs(FileSaveMsg),
    ConfigOpened(FileOpenMsg),

    ProductNameMsg(TextFieldMsg),
    ProductIdMsg(TextFieldMsg),
    StateTransitionMsg(TextFieldMsg),

    FwPaneMsg(usize, FwMsg),
    FwPaneRemove(usize),
    FwPaneUpdated(usize, FwConfig),
    FwPaneAdd,

    UseBackdoorToggle,
    AutoSaveToggle,
}

pub struct MainApp {
    product_name: TextField<String>,
    config_path: String,
    product_id: TextField<u16>,
    state_transition: TextField<u32>,
    auto_save: bool,
    config: Config,
    fw_configs: Vec<Component<FwPane>>,
    log: Vec<String>,
}

mod validate {
    pub fn product_name(value: &str) -> Option<String> {
        Some(value.to_string())
    }

    pub fn product_id(value: &str) -> Option<u16> {
        u16::from_str_radix(&value, 16).ok()
    }

    pub fn state_transition(value: &str) -> Option<u32> {
        u32::from_str_radix(&value, 16).ok()
    }
}

impl MainApp {
    pub fn new() -> Self {
        Self {
            product_name: TextField::new(validate::product_name, |x| x.to_string()),
            config_path: "".to_string(),
            product_id: TextField::new(validate::product_id, |x| format!("{:X}", x)),
            state_transition: TextField::new(validate::state_transition, |x| x.to_string()),
            auto_save: false,
            config: Default::default(),
            fw_configs: vec![],
            log: vec![Self::say_greeting()]
        }
    }

    fn say_greeting() -> String {
        // let's have a bit of fun ;)
        let lt = Local::now();
        if lt.hour() > 22 {
            "You should go to sleep!"
        } else if lt.hour() > 17 {
            "Good evening!"
        } else if lt.hour() > 12 {
            "Good afternoon!"
        } else if lt.hour() > 7 {
            "Good morning!"
        } else {
            "You are early... couldn't sleep?"
        }.to_string()
    }

    pub fn apply_config(&mut self, config: Config) {
        self.product_name.set(config.product_name.clone());
        self.product_id.set(config.product_id);
        self.state_transition.set(config.time_state_transition);
        self.config = config;
        for fw_config in &self.config.images {
            let comp = Component::new(FwPane::with_config(fw_config));
            self.fw_configs.push(comp);
        }
    }

    pub fn load_config(&mut self, path: String) {
        let path = Path::new(&path);
        self.fw_configs.clear();
        match Config::load_from_file(path) {
            Ok(config) => {
                self.apply_config(config);
                self.auto_save = true;
            }
            Err(err) => {
                println!("{:?}", err);
                // TODO: ...
                println!("TODO: error, print to log")
            }
        }
    }

    pub fn save_config(&self) {
        let config = self.config.clone();
        let path = self.config_path.clone();
        thread::spawn(move || {
            let path = Path::new(&path);
            config.save(path)
        });
    }

    pub fn config_changed(&self) {
        if self.auto_save {
            self.save_config();
        }
    }

    pub fn render_fws(&self) -> Node<Msg> {
        let ret = self.fw_configs.iter()
            .enumerate()
            .map(|(k, x)| {
                let component = x.mount().map(move |msg| Msg::FwPaneMsg(k, msg));
                let locked = x.lock();
                let remove: Node<Msg> = locked.remove.subscribe(move |_| Msg::FwPaneRemove(k)).into();
                let updated: Node<Msg> = locked.updated.subscribe(move |config| Msg::FwPaneUpdated(k, config)).into();
                let mut nodes = ArrayVec::from([component, remove, updated]);
                Node::new_from_iter(nodes.drain(..))
            });
        Node::new_from_iter(ret)
    }

    pub fn prop(&mut self, updated: (bool, Updated)) -> Updated {
        if updated.0 {
            self.config_changed();
        }
        updated.1
    }

    pub fn render_log(&self) -> ElementBuilder<Msg> {
        const JS: &'static str = r#"{
            let tgt = event.target;
            tgt.value = tgt.getAttribute('value');
            tgt.scrollTop = tgt.scrollHeight;
        }"#;
        Node::html().elem("textarea").class("form-control flex-fill mr-1").id("log-area")
            .attr("rows", "3").attr("value", self.log.join("\n"))
            .attr("readonly", "").js_event("render", JS)
    }
}

impl App for MainApp {
    fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated {
        println!("{:?}", msg);
        match msg {
            Msg::Open => {
                let dialog = FileOpenDialog::new("Open a config file", "")
                    .with_filter(FileFilter::new("gctmrg Config files").push("gctmrg"));
                ctx.dialog(dialog, Msg::ConfigOpened);
                Updated::no()
            }
            Msg::ConfigOpened(msg) => {
                if let FileOpenMsg::Selected(path) = msg {
                    self.config_path = path.clone();
                    self.load_config(path);
                }
                Updated::yes()
            }

            Msg::SaveAs => {
                let dialog = FileSaveDialog::new("Save config file as...", "config.json.gctmrg")
                    .with_filter(FileFilter::new("gctmrg Config files").push("gctmrg"));
                ctx.dialog(dialog, Msg::ConfigSavedAs);
                Updated::no()
            }
            Msg::ConfigSavedAs(msg) => {
                if let FileSaveMsg::SaveTo(path) = msg {
                    self.config_path = path.clone();
                    self.save_config();
                }
                Updated::yes()
            }

            Msg::ProductNameMsg(msg) => {
                let ret = self.product_name.update(&mut self.config.product_name, msg);
                self.prop(ret)
            }

            Msg::ProductIdMsg(msg) => {
                let ret = self.product_id.update(&mut self.config.product_id, msg);
                self.prop(ret)
            }

            Msg::StateTransitionMsg(msg) => {
                let ret = self.state_transition.update(&mut self.config.time_state_transition, msg);
                self.prop(ret)
            }
            Msg::UseBackdoorToggle => {
                self.config.use_backdoor = !self.config.use_backdoor;
                self.config_changed();
                Updated::yes()
            }
            Msg::AutoSaveToggle => {
                self.auto_save = !self.auto_save;
                self.config_changed();
                Updated::yes()
            }
            Msg::FwPaneMsg(k, msg) => {
                let ctx = ctx.map(move |x| Msg::FwPaneMsg(k, x));
                self.fw_configs[k].update(msg, ctx)
            }
            Msg::FwPaneRemove(k) => {
                self.fw_configs.remove(k);
                self.config.images.remove(k);
                self.config_changed();
                Updated::yes()
            }
            Msg::FwPaneUpdated(k, fw_config) => {
                self.config.images[k] = fw_config;
                self.config_changed();
                Updated::yes()
            }
            Msg::FwPaneAdd => {
                self.config.images.push(Default::default());
                self.fw_configs.push(Component::new(FwPane::new()));
                self.config_changed();
                Updated::yes()
            }
        }
    }
}

impl Render for MainApp {
    type Message = Msg;

    fn render(&self) -> Node<Self::Message> {
        use greenhorn::html;

        html!(
            <div .main-app .d-flex .flex-column>
                // path to config file
                <div class="d-flex flex-row align-items-center my-2">
                    <div class="custom-control custom-switch mx-1 col-auto">
                        {checkbox(self.auto_save, || Msg::AutoSaveToggle)
                            .class("custom-control-input").id("auto-save-toggle")}
                        <label class="custom-control-label" for="auto-save-toggle">{"Auto Save"}</>
                    </>
                    <span class="form-control mx-1" readonly="">{&self.config_path}</>
                    <button type="button" class="btn btn-secondary mx-1 col-auto"
                        @click={|_| Msg::Open}>{"Open"}</>
                    <button type="button" class="btn btn-secondary mx-1 col-auto"
                        @click={|_| Msg::SaveAs}>{"Save As"}</>
                </>

                // row with product ID + Product name
                <div class="d-flex flex-row align-items-center my-2">
                    <span class="col-3">{"Product ID"}</>
                    <div class="input-group col-3" #product-id-div>
                        <div class="input-group-prepend">
                            <span class="input-group-text">{"0x"}</span>
                        </>
                        {self.product_id.render().class("form-control")
                            .attr("placeholder", "e.g. 0xABCD").build().map(Msg::ProductIdMsg)}
                    </>
                    <span class="col-3">{"Product Name"}</>
                    {self.product_name.render().class("col-3 form-control")
                        .attr("placeholder", "e.g. Nimbus2000").build().map(Msg::ProductNameMsg)}
                </>

                // row with state transition and "use backdoor"
                <div class="d-flex flex-row align-items-center my-2">
                    <span class="col-3">{"State Transition Time [ms]"}</>
                    {self.state_transition.render() .class("col-3 form-control")
                        .attr("placeholder", "in ms").build().map(Msg::StateTransitionMsg)}
                    <div class="col-6 px-5 custom-control custom-checkbox">
                        {checkbox(self.config.use_backdoor, || Msg::UseBackdoorToggle)
                            .class("custom-control-input").id("use-backdoor")}
                        <label class="custom-control-label" for="use-backdoor">
                        {"Use Backdoor"}
                        </>
                    </>
                </>

                <div #fws-container class="my-2 d-flex flex-column">
                    <div #fws-title-bar class="d-flex flex-row justify-content-between align-items-center">
                        <span class="mx-2" #fws-title>{"Firmware Images"}</>
                        <button class="btn btn-secondary" @click={|_| Msg::FwPaneAdd}>
                            <i class="icofont-ui-add" />
                        </>
                    </>
                    <div #fws-row class="d-flex flex-row">{self.render_fws()}</>                    
                </>

                // main action buttons
                <div class="d-flex flex-row my-2">
                    {self.render_log()}
                    <button type="button" class="btn btn-secondary mx-1 main-btn">{"Merge"}</>
                    <button type="button" class="btn btn-secondary mx-1 main-btn">{"Release"}</>
                    <button type="button" class="btn btn-primary ml-1 main-btn">{"Generate Script"}</>
                </>
            </>
        )
        .into()
    }
}
