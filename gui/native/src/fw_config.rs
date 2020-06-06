use greenhorn::prelude::*;
use greenhorn::html;
use merge_tool::config::FwConfig;
use crate::text_field::{TextField, TextFieldMsg};
use greenhorn::dialog::{FileOpenDialog, FileOpenMsg, FileFilter};

#[derive(Debug)]
pub enum FwMsg {
    Remove,
    UpdateConfig(FwConfig),
    FwIdMsg(TextFieldMsg),
    AppPathMsg(TextFieldMsg),
    BtlPathMsg(TextFieldMsg),
    OpenApp,
    OpenBtl,
    OpenAppDialog(FileOpenMsg),
    OpenBtlDialog(FileOpenMsg),
    FwIdChanged(u8),
}

pub struct FwPane {
    config: FwConfig,
    pub updated: Event<FwConfig>,
    pub remove: Event<()>,
    fw_id: TextField<u8>,
    btl_path: TextField<String>,
    app_path: TextField<String>,
}

impl Default for FwPane {
    fn default() -> Self {
        Self {
            config: Default::default(),
            updated: Default::default(),
            remove: Default::default(),
            fw_id: TextField::new(|x| u8::from_str_radix(x, 16).ok(),
                                  |x| format!("{:X}", x),
                                  1).class("form-control flex-fill"),
            btl_path: TextField::new(|x| Some(x.to_string()),
                                     |x| x.clone(),
                                     String::new()).class("form-control flex-fill"),
            app_path: TextField::new(|x| Some(x.to_string()),
                                     |x| x.clone(),
                                     String::new()).class("form-control flex-fill"),
        }
    }
}

impl FwPane {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_config(config: &FwConfig) -> Self {
        let mut ret = Self::new();
        ret.config = config.clone();
        ret
    }

    fn open_hex_file(&self) -> FileOpenDialog {
        FileOpenDialog::new("Open hex file...", "~")
            .with_filter(FileFilter::new("hex files")
                .push("s37")
                .push("hex"))
    }

    fn make_path_relative(&self, path: &str) -> String {
        // TODO: ...
        path.into()
    }
}

impl App for FwPane {
    fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated {
        match msg {
            FwMsg::Remove => {
                ctx.emit(&self.remove, ());
                Updated::no()
            },

            FwMsg::UpdateConfig(config) => {
                self.config = config;
                Updated::yes()
            }
            FwMsg::FwIdMsg(msg) => self.fw_id.update(msg, &ctx),
            FwMsg::OpenApp => {
                ctx.dialog(self.open_hex_file(), FwMsg::OpenAppDialog);
                Updated::no()
            }
            FwMsg::OpenBtl => {
                ctx.dialog(self.open_hex_file(), FwMsg::OpenBtlDialog);
                Updated::no()
            }
            FwMsg::OpenAppDialog(msg) => {
                if let FileOpenMsg::Selected(path) = msg {
                    let path = self.make_path_relative(&path);
                    self.app_path.set(path);
                    Updated::yes()
                } else {
                    Updated::no()
                }
            }
            FwMsg::OpenBtlDialog(msg) => {
                if let FileOpenMsg::Selected(path) = msg {
                    let path = self.make_path_relative(&path);
                    self.btl_path.set(path);
                    Updated::yes()
                } else {
                    Updated::no()
                }
            }
            FwMsg::AppPathMsg(msg) => self.app_path.update(msg, &ctx),
            FwMsg::BtlPathMsg(msg) => self.btl_path.update(msg, &ctx),

            FwMsg::FwIdChanged(id) => {
                self.config.fw_id = id;
                ctx.emit(&self.updated, self.config.clone());
                Updated::no()
            }
        }
    }
}

impl Render for FwPane {
    type Message = FwMsg;

    fn render(&self) -> Node<Self::Message> {
        html!( <div #fw-config >
                // row with product ID + Product name
                <div class="d-flex flex-row align-items-center my-2">
                    <span class="col-4">{"Firmware ID"}</>
                    {self.fw_id.render().build().map(FwMsg::FwIdMsg)}
                    {self.fw_id.change_event().subscribe(FwMsg::FwIdChanged)}
                </>
                <div class="d-flex flex-row align-items-center my-2">
                    <span class="col-4">{"App Path"}</>
                    {self.app_path.render().build().map(FwMsg::AppPathMsg)}
                    <button type="button" class="btn btn-secondary mx-1"
                        @click={|_| FwMsg::OpenApp}>{"..."}</>
                </>
                <div class="d-flex flex-row align-items-center my-2">
                    <span class="col-4">{"Btl Path"}</>
                    {self.btl_path.render().build().map(FwMsg::BtlPathMsg)}
                    <button type="button" class="btn btn-secondary mx-1"
                        @click={|_| FwMsg::OpenBtl}>{"..."}</>
                </>
                <div class="flex-fill"/>
                <div class="d-flex flex-row justify-content-end my-2">
                    <button type="button" class="btn btn-secondary"
                        @click={|_| FwMsg::Remove}>{"Remove"}</>
                </div>
        </> ).into()
    }
}
