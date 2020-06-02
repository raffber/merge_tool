use std::thread;
use std::path::Path;

use greenhorn::components::checkbox;
use greenhorn::dialog::{FileFilter, FileOpenDialog, FileOpenMsg};
use greenhorn::prelude::*;

use merge_tool::config::Config;

use crate::text_field::{TextField, TextFieldMsg};

#[derive(Debug)]
pub enum Msg {
    OpenConfig,
    ConfigOpened(FileOpenMsg),

    ConfigPathMsg(TextFieldMsg),
    ConfigPathChanged(String),

    ProductNameMsg(TextFieldMsg),
    ProductNameChanged(String),

    ProductIdMsg(TextFieldMsg),
    ProductIdChanged(u16),

    StateTransitionMsg(TextFieldMsg),
    StateTransitionChanged(u32),

    UseBackdoorToggle,
    AutoSaveToggle,
}

pub struct MainApp {
    product_name: TextField<String>,
    config_path: TextField<String>,
    product_id: TextField<u16>,
    state_transition: TextField<u32>,
    auto_save: bool,
    config: Config,
}

mod validate {
    pub fn product_name(value: &str) -> Option<String> {
        Some(value.to_string())
    }

    pub fn config_path(value: &str) -> Option<String> {
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
            product_name: TextField::new(validate::product_name, |x| x.to_string(), "".to_string())
                .class("col-3 form-control").placeholder("e.g. Nimbus2000"),
            config_path: TextField::new(validate::config_path, |x| x.to_string(), "".to_string())
                .class("col mx-1 form-control").placeholder("Path to config file..."),
            product_id: TextField::new(validate::product_id, |x| x.to_string(), 0)
                .class("col-3 form-control").placeholder("e.g. 0xABCD"),
            state_transition: TextField::new(validate::state_transition, |x| x.to_string(), 0)
                .class( "col-3 form-control").placeholder( "in ms"),
            auto_save: false,
            config: Default::default(),
        }
    }

    pub fn apply_config(&mut self, config: Config) {
        self.product_name.set(config.product_name.clone());
        self.product_id.set(config.product_id);
        self.state_transition.set(config.time_state_transition);
        self.config = config;
    }

    pub fn load_config(&mut self, path: String) {
        let path = Path::new(&path);
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

    pub fn config_changed(&self) {
        if self.auto_save {
            let config = self.config.clone();
            let path = self.config_path.get().clone();
            thread::spawn(move || {
                let path = Path::new(&path);
                config.save(path)
            });
        }
    }
}

impl App for MainApp {
    fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated {
        println!("{:?}", msg);
        match msg {
            Msg::OpenConfig => {
                let dialog = FileOpenDialog::new("Open a config file", "~")
                    .with_filter(FileFilter::new("GCTBtl Config files").push("gctmrg"));
                ctx.dialog(dialog, Msg::ConfigOpened)
            }

            Msg::ConfigOpened(msg) => {
                if let FileOpenMsg::Selected(path) = msg {
                    self.config_path.set(path.clone());
                    self.load_config(path);
                }
            }
            Msg::ConfigPathMsg(msg) => {
                self.config_path.update(msg, &ctx);
            },

            Msg::ProductNameMsg(msg) => {
                self.product_name.update(msg, &ctx);
            },
            Msg::ProductNameChanged(value) => {
                self.config.product_name = value;
                self.config_changed();
            }

            Msg::ConfigPathChanged(value) => {
                self.load_config(value);
            }
            Msg::ProductIdMsg(msg) => {
                self.product_id.update(msg, &ctx);
            },
            Msg::ProductIdChanged(value) => {
                self.config.product_id = value;
                self.config_changed();
            }

            Msg::StateTransitionMsg(msg) => {
                self.state_transition.update(msg, &ctx);
            },
            Msg::StateTransitionChanged(value) => {
                self.config.time_state_transition = value;
                self.config_changed();
            }
            Msg::UseBackdoorToggle => {
                self.config.use_backdoor = !self.config.use_backdoor;
                self.config_changed();
            },
            Msg::AutoSaveToggle => {
                self.auto_save = !self.auto_save;
                self.config_changed();
            },
        }
        Updated::yes()
    }
}

impl Render for MainApp {
    type Message = Msg;

    fn render(&self) -> Node<Self::Message> {
        use greenhorn::html;

        html!(
            <div .main-app .container-fluid>
                // path to config file
                <div class="row align-items-center my-2">
                    <div class="custom-control custom-switch mx-1 col-auto">
                        {checkbox(self.auto_save, || Msg::AutoSaveToggle)
                            .class("custom-control-input").id("auto-save-toggle")}
                        <label class="custom-control-label" for="auto-save-toggle">{"Auto Save"}</>
                    </>
                    {self.config_path.render().build().map(Msg::ConfigPathMsg)}
                    {self.config_path.change_event().subscribe(Msg::ConfigPathChanged)}
                    <button type="button" class="btn btn-secondary mx-1 col-auto"
                        @click={|_| Msg::OpenConfig}>{"Open"}</>
                </>

                // row with product ID + Product name
                <div class="row align-items-center my-2">
                    <span class="col-3">{"Product ID"}</>
                    {self.product_id.render().build().map(Msg::ProductIdMsg)}
                    {self.product_id.change_event().subscribe(Msg::ProductIdChanged)}
                    <span class="col-3">{"Product Name"}</>
                    {self.product_name.render().build().map(Msg::ProductNameMsg)}
                    {self.product_name.change_event().subscribe(Msg::ProductNameChanged)}
                </>

                // row with state transition and "use backdoor"
                <div class="row align-items-center my-2">
                    <span class="col-3">{"State Transition Time"}</>
                    {self.state_transition.render().build().map(Msg::StateTransitionMsg)}
                    {self.state_transition.change_event().subscribe(Msg::StateTransitionChanged)}
                    <div class="col-6 px-5 custom-control custom-checkbox">
                        {checkbox(self.config.use_backdoor, || Msg::UseBackdoorToggle)
                            .class("custom-control-input").id("use-backdoor")}
                        <label class="custom-control-label" for="use-backdoor">
                        {"Use Backdoor"}
                        </>
                    </>                    
                </>

                // main action buttons
                <div #main-button-row>                    
                    <button type="button" class="btn btn-secondary mx-1">{"Merge"}</>
                    <button type="button" class="btn btn-secondary mx-1">{"Release"}</>
                    <button type="button" class="btn btn-primary mx-1">{"Generate Script"}</>
                </>
            </>
        ).into()
    }
}
