use greenhorn::prelude::*;
use greenhorn::components::{checkbox, TextInput, TextInputMsg};
use greenhorn::dialog::{FileOpenDialog, FileOpenMsg, FileFilter};

use merge_tool::config::Config;
use std::path::Path;
use merge_tool::Error;
use std::panic;
use backtrace::Backtrace;

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
}

pub struct MainApp {
    product_name: TextInput,
    config_path: TextInput,
    product_id: TextInput,
    state_transition: TextInput,
    config: Config,
}

impl MainApp {
    pub fn new() -> Self {
        Self {
            product_name: Default::default(),
            config_path: Default::default(),
            product_id: Default::default(),
            state_transition: Default::default(),
            config: Default::default(),
        }
    }

    pub fn apply_config(&mut self, config: Config) {
        self.config = config;
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
            },

            Msg::ConfigOpened(msg) => {
                println!("{:?}", msg);
                if let FileOpenMsg::Selected(path) = msg {
                    self.config_path.set(path.clone());
                    let path = Path::new(&path);
                    match Config::load_from_file(path) {
                        Ok(config) => {
                            self.apply_config(config);
                        },
                        Err(err) => {
                            println!("{:?}", err);
                            // TODO: ...
                            println!("TODO: error, print to log")
                        },
                    }
                }
            },
            Msg::ConfigPathMsg(msg) => self.config_path.update(msg, &ctx),

            Msg::ProductNameMsg(msg) => self.product_name.update(msg, &ctx),
            Msg::ProductNameChanged(_) => {}

            Msg::ConfigPathChanged(_) => {
                println!("reload!!");
            }
            Msg::ProductIdMsg(msg) => self.product_id.update(msg, &ctx),
            Msg::ProductIdChanged(_) => {}

            Msg::StateTransitionMsg(_) => {}
            Msg::StateTransitionChanged(_) => {}
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
                        <input type="checkbox" class="custom-control-input" id="auto-save-toggle" />
                        <label class="custom-control-label" for="auto-save-toggle">{"Auto Save"}</>
                    </>
                    {self.config_path.render(Msg::ConfigPathMsg).class("col mx-1 form-control")
                        .attr("placeholder", "Path to config file...")
                        .on("keyup", Msg::ConfigPathChanged)}
                    <button type="button" class="btn btn-secondary mx-1 col-auto" @click={|_| Msg::OpenConfig}>{"Open"}</>
                </>

                // row with product ID + Product name
                <div class="row align-items-center my-2">
                    <span class="col-3">{"Product ID"}</>
                    {self.product_id.render(Msg::ProductIdMsg)
                        .class("col-3 form-control")
                        .attr("placeholder", "e.g. 0xABCD")
                        .on("keyup", Msg::ProductIdChanged)}
                    <span class="col-3">{"Product Name"}</>
                    {self.product_name.render(Msg::ProductNameMsg)
                        .class("col-3 form-control")
                        .attr("placeholder", "e.g. Nimbus2000")
                        .on("keyup", Msg::ProductNameChanged)}
                </>

                // row with state transition and "use backdoor"
                <div class="row align-items-center my-2">
                    <span class="col-3">{"State Transition Time"}</>
                    {self.state_transition.render(Msg::StateTransitionMsg)
                        .class("col-3 form-control")
                        .attr("placeholder", "in ms")
                        .on("keyup", Msg::StateTransitionChanged)}
                    <div class="col-6 px-5 custom-control custom-checkbox">
                        <input type="checkbox" class="custom-control-input" id="use-backdoor" />
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
