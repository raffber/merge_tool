use greenhorn::prelude::*;
use greenhorn::components::{checkbox, TextInput, TextInputMsg};
use greenhorn::dialog::{FileOpenDialog, FileOpenMsg, FileFilter};

pub enum Msg {
    OpenConfig,
    ConfigOpened(FileOpenMsg),
    ConfigPathChanged(TextInputMsg),
}

pub struct MainApp {
    product_name: TextInput,
    config_path: TextInput,
}

impl MainApp {
    pub fn new() -> Self {
        Self {
            product_name: Default::default(),
            config_path: Default::default()
        }
    }
}

impl App for MainApp {
    fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated {
        match msg {
            Msg::OpenConfig => {
                let dialog = FileOpenDialog::new("Open a config file", "~")
                    .with_filter(FileFilter::new("GCTBtl Config files").push("gctmrg"));
                ctx.dialog(dialog, Msg::ConfigOpened)
            },

            Msg::ConfigOpened(msg) => {
                println!("{:?}", msg)
            },
            Msg::ConfigPathChanged(msg) => {}
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
                <div class="row align-items-center my-2">
                    <div class="custom-control custom-switch mx-1 col-auto">
                        <input type="checkbox" class="custom-control-input" id="auto-save-toggle" />
                        <label class="custom-control-label" for="auto-save-toggle">{"Auto Save"}</>
                    </>
                    {self.config_path.render(Msg::ConfigPathChanged).class("col mx-1 form-control")
                        .attr("placeholder", "Path to config file...")}
                    <button type="button" class="btn btn-secondary mx-1 col-auto" @click={|_| Msg::OpenConfig}>{"Open"}</>
                </>
                <div class="row align-items-center my-2">
                    <span class="col-3">{"Product ID"}</>
                    <input type="text" class="col-3 form-control" placeholder="e.g. 0xABCD"/>
                    <span class="col-3">{"Product Name"}</>
                    <input type="text" class="col-3 form-control" placeholder="e.g. Nimbus2000"/>
                </>
                <div class="row align-items-center my-2">
                    <span class="col-3">{"State Transition Time"}</>
                    <input type="text" class="col-3 form-control" placeholder="in ms"/>
                    <div class="col-6 px-5 custom-control custom-checkbox">
                        <input type="checkbox" class="custom-control-input" id="use-backdoor" />
                        <label class="custom-control-label" for="use-backdoor">
                        {"Use Backdoor"}
                        </>
                    </>                    
                </>
                <div #main-button-row>                    
                    <button type="button" class="btn btn-secondary mx-1">{"Merge"}</>
                    <button type="button" class="btn btn-secondary mx-1">{"Release"}</>
                    <button type="button" class="btn btn-primary mx-1">{"Generate Script"}</>
                </>
            </>
        ).into()
    }
}
