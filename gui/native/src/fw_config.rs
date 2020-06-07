use greenhorn::prelude::*;
use greenhorn::html;
use merge_tool::config::{FwConfig, HexFileFormat};
use greenhorn::dialog::{FileOpenDialog, FileOpenMsg, FileFilter};
use crate::address_pane::{AddressPane, AddressPaneMsg};
use greenhorn::components::checkbox;
use std::str::FromStr;
use crate::lean_field::{LeanMsg, LeanField};

#[derive(Debug)]
pub enum FwMsg {
    Remove,
    UpdateConfig(FwConfig),
    FwIdMsg(LeanMsg),
    AppPathMsg(LeanMsg),
    BtlPathMsg(LeanMsg),
    OpenApp,
    OpenBtl,
    OpenAppDialog(FileOpenMsg),
    OpenBtlDialog(FileOpenMsg),
    AppAddrMsg(AddressPaneMsg),
    BtlAddrMsg(AddressPaneMsg),
    IncludeToggle,
    PageSizeMsg(LeanMsg),
    WordAddressingToggle,
    TimeDataSendMsg(LeanMsg),
    TimeSendDoneMsg(LeanMsg),
    TimeLeaveMsg(LeanMsg),
    TimeEraseMsg(LeanMsg),
    HexSelectChanged(JsonValue),
    HeaderOffsetMsg(LeanMsg),
}

pub struct FwPane {
    config: FwConfig,
    pub updated: Event<FwConfig>,
    pub remove: Event<()>,
    fw_id: LeanField<u8>,
    btl_path: LeanField<String>,
    app_path: LeanField<String>,
    app_addr: AddressPane,
    btl_addr: AddressPane,
    include_id: String,
    page_size: LeanField<u64>,
    word_addressing_id: String,
    header_offset: LeanField<u64>,
    time_data_send: LeanField<u32>,
    time_send_done: LeanField<u32>,
    time_leave: LeanField<u32>,
    time_erase: LeanField<u32>,
}

impl Default for FwPane {
    fn default() -> Self {
        Self {
            config: Default::default(),
            updated: Default::default(),
            remove: Default::default(),
            fw_id: LeanField::new(|x| u8::from_str_radix(x, 16).ok(),
                                  |x| format!("{:X}", x)),
            btl_path: LeanField::new(|x| Some(x.to_string()),
                                     |x| x.clone()),
            app_path: LeanField::new(|x| Some(x.to_string()),
                                     |x| x.clone()),
            app_addr: Default::default(),
            btl_addr: Default::default(),
            include_id: format!("{}", Id::new().data()),
            word_addressing_id: format!("{}", Id::new().data()),
            header_offset: LeanField::new(|x| u64::from_str_radix(x, 16).ok(),
                           |x| format!("{:X}", x))
,
            time_data_send: Self::make_time_field(),
            time_send_done: Self::make_time_field(),
            time_leave: Self::make_time_field(),
            page_size: LeanField::new(|x| u64::from_str_radix(x, 16).ok(),
                                      |x| format!("{:X}", x)),
            time_erase: Self::make_time_field(),
        }
    }
}

impl FwPane {
    pub fn new() -> Self {
        let mut ret : FwPane = Default::default();
        ret.config.device_config.page_size = 2;
        ret
    }

    fn make_time_field() -> LeanField<u32> {
        LeanField::new(|x| u32::from_str(x).ok(),
                       |x| x.to_string())
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

    fn emit(&self, ctx: &Context<FwMsg>) {
        ctx.emit(&self.updated, self.config.clone());
    }

    fn prop(&mut self, ctx: &Context<FwMsg>, ret: (bool, Updated)) -> Updated {
        if ret.0 {
            self.emit(&ctx);
        }
        ret.1
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
            FwMsg::FwIdMsg(msg) => {
                let ret = self.fw_id.update(&mut self.config.fw_id, msg);
                self.prop(&ctx, ret)
            },
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
            FwMsg::AppPathMsg(msg) => {
                let ret = self.app_path.update(&mut self.config.app_path, msg);
                self.prop(&ctx, ret)
            },
            FwMsg::BtlPathMsg(msg) =>{
                let ret = self.btl_path.update(&mut self.config.btl_path, msg);
                self.prop(&ctx, ret)
            }

            FwMsg::AppAddrMsg(msg) => {
                let (changed, ret) = self.app_addr.update(msg, &mut self.config.app_address);
                if changed {
                    self.emit(&ctx);
                }
                ret
            },

            FwMsg::BtlAddrMsg(msg) => {
                let (changed, ret) = self.btl_addr.update(msg, &mut self.config.btl_address);
                if changed {
                    self.emit(&ctx);
                }
                ret
            },

            FwMsg::IncludeToggle => {
                self.config.include_in_script = !self.config.include_in_script;
                self.emit(&ctx);
                Updated::yes()
            }
            FwMsg::WordAddressingToggle => {
                self.config.device_config.word_addressing = !self.config.device_config.word_addressing;
                self.emit(&ctx);
                Updated::yes()
            }
            FwMsg::PageSizeMsg(msg) => {
                let ret = self.page_size.update(&mut self.config.device_config.page_size, msg);
                self.prop(&ctx, ret)
            },

            FwMsg::TimeDataSendMsg(msg) => {
                let ret = self.time_data_send.update(&mut self.config.timings.data_send, msg);
                self.prop(&ctx, ret)
            },

            FwMsg::TimeSendDoneMsg(msg) =>{
                let ret = self.time_send_done.update(&mut self.config.timings.data_send_done, msg);
                self.prop(&ctx, ret)
            }

            FwMsg::TimeLeaveMsg(msg) => {
                let ret = self.time_leave.update(&mut self.config.timings.leave_btl, msg);
                self.prop(&ctx, ret)
            },

            FwMsg::TimeEraseMsg(msg) =>{
                let ret = self.time_erase.update(&mut self.config.timings.erase_time, msg);
                self.prop(&ctx, ret)
            },

            FwMsg::HexSelectChanged(value) => {
                let idx: u32 = serde_json::from_value(value).unwrap();
                if idx == 0 {
                    self.config.hex_file_format = HexFileFormat::IntelHex;
                } else if idx == 1 {
                    self.config.hex_file_format = HexFileFormat::SRecord;
                } else {
                    panic!();
                }
                self.emit(&ctx);
                Updated::no()
            }
            FwMsg::HeaderOffsetMsg(msg) =>{
                let ret = self.header_offset.update(&mut self.config.header_offset, msg);
                self.prop(&ctx, ret)
            }
        }
    }
}

impl Render for FwPane {
    type Message = FwMsg;

    fn render(&self) -> Node<Self::Message> {
        html!(<div #fw-config>
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-4">{"Firmware ID"}</>
                    {self.fw_id.render().class("form-control flex-fill").build().map(FwMsg::FwIdMsg)}
                    <div class="custom-control custom-switch include-script-checkbox ml-2">
                        {checkbox(self.config.include_in_script, || FwMsg::IncludeToggle)
                            .class("custom-control-input").id(self.include_id.clone())}
                        <label class="custom-control-label" for={self.include_id.clone()}>{"Include in Script"}</>
                    </>
                </>
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-4">{"App Path"}</>
                    {self.app_path.render().class("form-control flex-fill")
                        .build().map(FwMsg::AppPathMsg)}
                    <button type="button" class="btn btn-secondary ml-2"
                        @click={|_| FwMsg::OpenApp}>{"..."}</>
                </>
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-4">{"Btl Path"}</>
                    {self.btl_path.render().class("form-control flex-fill")
                        .build().map(FwMsg::BtlPathMsg)}
                    <button type="button" class="btn btn-secondary ml-2"
                        @click={|_| FwMsg::OpenBtl}>{"..."}</>
                </>
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-4">{"App Address"}</>
                    {self.app_addr.render().map(FwMsg::AppAddrMsg)}
                </div>
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-4">{"Btl Address"}</>
                    {self.btl_addr.render().map(FwMsg::BtlAddrMsg)}
                </div>
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-4">{"Page Size"}</>
                    {self.page_size.render().class("form-control flex-fill")
                        .attr("placeholder", "in hex").build().map(FwMsg::PageSizeMsg)}
                    <div class="custom-control custom-checkbox mx-2 word-addressing-checkbox">
                        {checkbox(self.config.device_config.word_addressing, || FwMsg::WordAddressingToggle)
                            .class("custom-control-input").id(self.word_addressing_id.clone())}
                        <label class="custom-control-label" for={self.word_addressing_id.clone()}>
                            {"Word Addresssing"}
                        </>
                    </div>
                </div>
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-4">{"Hex Format"}</>
                    <select class="form-control"
                        $change="app.send(event.target, event.target.selectedIndex)"
                        @rpc={FwMsg::HexSelectChanged}>
                      <option>{"Intel Hex *.hex"}</option>
                      <option>{"S-Record *.s37"}</option>
                    </select>
                </>
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-4">{"Header Offset"}</>
                    {self.header_offset.render().class("form-control flex-fill")
                        .attr("placeholder", "in hex").build().map(FwMsg::HeaderOffsetMsg)}
                </div>

                // timings
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-6">{"Time between data send"}</>
                    {self.time_data_send.render().class("form-control flex-fill")
                        .attr("placeholder", "in ms").build().map(FwMsg::TimeDataSendMsg)}
                </div>
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-6">{"Time after send done"}</>
                    {self.time_send_done.render().class("form-control flex-fill")
                        .attr("placeholder", "in ms").build().map(FwMsg::TimeSendDoneMsg)}
                </div>
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-6">{"Time to leave bootloader"}</>
                    {self.time_leave.render().class("form-control flex-fill")
                        .attr("placeholder", "in ms").build().map(FwMsg::TimeLeaveMsg)}
                </div>
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-6">{"Erase Time"}</>
                    {self.time_erase.render().class("form-control flex-fill")
                        .attr("placeholder", "in ms").build().map(FwMsg::TimeEraseMsg)}
                </div>

                // fill up the rest of the space
                <div class="flex-fill"/>

                // and the remove button...
                <div class="d-flex flex-row justify-content-end my-2">
                    <button type="button" class="btn btn-secondary"
                        @click={|_| FwMsg::Remove}>{"Remove"}</>
                </div>
        </> ).into()
    }
}
