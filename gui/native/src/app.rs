use std::panic;
use std::path::Path;

use backtrace::Backtrace;
use greenhorn::components::{checkbox, TextInput, TextInputMsg};
use greenhorn::dialog::{FileFilter, FileOpenDialog, FileOpenMsg};
use greenhorn::prelude::*;

use merge_tool::config::Config;
use merge_tool::Error;

#[derive(Debug)]
pub enum Msg {
    OpenConfig,
    ConfigOpened(FileOpenMsg),

    ConfigPathMsg(TextInputMsg),
    ConfigPathChanged(DomEvent),

    ProductNameMsg(TextInputMsg),
    ProductNameChanged(DomEvent),

    ProductIdMsg(TextInputMsg),
    ProductIdChanged(DomEvent),

    StateTransitionMsg(TextInputMsg),
    StateTransitionChanged(DomEvent),

    UseBackdoorToggle,
    AutoSaveToggle,
}

pub struct MainApp {
    product_name: TextInput,
    config_path: TextInput,
    product_id: TextInput,
    product_id_valid: bool,
    state_transition: TextInput,
    state_transition_valid: bool,
    auto_save: bool,
    config: Config,
}

impl MainApp {
    pub fn new() -> Self {
        Self {
            product_name: Default::default(),
            config_path: Default::default(),
            product_id: Default::default(),
            state_transition: Default::default(),
            state_transition_valid: true,
            auto_save: false,
            product_id_valid: true,
            config: Default::default(),
        }
    }

    pub fn apply_config(&mut self, config: Config) {
        self.product_name.set(config.product_name.clone());
        self.product_id.set(format!("{:#X}", &config.product_id));
        self.state_transition.set(format!("{}", &config.time_state_transition));
        self.config = config;
    }

    pub fn load_config(&mut self, path: String, ctx: &Context<Msg>) {
        let path = Path::new(&path);
        match Config::load_from_file(path) {
            Ok(config) => {
                self.apply_config(config);
            }
            Err(err) => {
                println!("{:?}", err);
                // TODO: ...
                println!("TODO: error, print to log")
            }
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
                    self.load_config(path, &ctx);
                }
            }
            Msg::ConfigPathMsg(msg) => self.config_path.update(msg, &ctx),

            Msg::ProductNameMsg(msg) => self.product_name.update(msg, &ctx),
            Msg::ProductNameChanged(evt) => {
                self.config.product_name = evt.target_value().get_text().unwrap();
            }

            Msg::ConfigPathChanged(evt) => {
                let path = evt.target_value().get_text().unwrap();
                self.load_config(path, &ctx);
            }
            Msg::ProductIdMsg(msg) => self.product_id.update(msg, &ctx),
            Msg::ProductIdChanged(evt) => {
                let value = evt.target_value().get_text().unwrap();
                if let Ok(value) = u16::from_str_radix(&value, 16) {
                    self.product_id_valid = true;
                    self.config.product_id = value;
                } else {
                    self.product_id_valid = false;
                }
            }

            Msg::StateTransitionMsg(msg) => self.state_transition.update(msg, &ctx),
            Msg::StateTransitionChanged(evt) => {
                let value = evt.target_value().get_text().unwrap();
                if let Ok(value) = u32::from_str_radix(&value, 16) {
                    self.state_transition_valid = true;
                    self.config.time_state_transition = value;
                } else {
                    self.state_transition_valid = false;
                }
            }
            Msg::UseBackdoorToggle => self.config.use_backdoor = !self.config.use_backdoor,
            Msg::AutoSaveToggle => self.auto_save = !self.auto_save,
        }
        Updated::yes()
    }
}

impl Render for MainApp {
    type Message = Msg;

    fn render(&self) -> Node<Self::Message> {
        use greenhorn::html;

        let mut state_transition = self.state_transition
            .render(Msg::StateTransitionMsg)
            .class("col-3 form-control")
            .attr("placeholder", "in ms")
            .on("keyup", Msg::StateTransitionChanged);
        if !self.state_transition_valid {
            state_transition = state_transition.class("is-invalid");
        }

        let mut product_id = self.product_id
            .render(Msg::ProductIdMsg)
            .class("col-3 form-control")
            .attr("placeholder", "e.g. 0xABCD")
            .on("keyup", Msg::ProductIdChanged);
        if !self.product_id_valid {
            product_id = product_id.class("is-invalid");
        }

        html!(
            <div .main-app .container-fluid>
                // path to config file
                <div class="row align-items-center my-2">
                    <div class="custom-control custom-switch mx-1 col-auto">
                        {checkbox(self.auto_save, || Msg::AutoSaveToggle)
                            .class("custom-control-input").id("auto-save-toggle")}
                        <label class="custom-control-label" for="auto-save-toggle">{"Auto Save"}</>
                    </>
                    {self.config_path.render(Msg::ConfigPathMsg).class("col mx-1 form-control")
                        .attr("placeholder", "Path to config file...")
                        .on("keyup", Msg::ConfigPathChanged)}
                    <button type="button" class="btn btn-secondary mx-1 col-auto"
                        @click={|_| Msg::OpenConfig}>{"Open"}</>
                </>

                // row with product ID + Product name
                <div class="row align-items-center my-2">
                    <span class="col-3">{"Product ID"}</>
                    {product_id}
                    <span class="col-3">{"Product Name"}</>
                    {self.product_name.render(Msg::ProductNameMsg)
                        .class("col-3 form-control")
                        .attr("placeholder", "e.g. Nimbus2000")
                        .on("keyup", Msg::ProductNameChanged)}
                </>

                // row with state transition and "use backdoor"
                <div class="row align-items-center my-2">
                    <span class="col-3">{"State Transition Time"}</>
                    {state_transition}
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
