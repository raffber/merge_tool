use greenhorn::prelude::*;
use greenhorn::html;
use merge_tool::config::{FwConfig, AddressRange, HexFileFormat};
use crate::text_field::{TextField, TextFieldMsg};
use greenhorn::dialog::{FileOpenDialog, FileOpenMsg, FileFilter};
use crate::address_pane::{AddressPane, AddressPaneMsg};
use greenhorn::components::checkbox;
use std::str::FromStr;

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
    AppAddrMsg(AddressPaneMsg),
    AppAddrUpdated(AddressRange),
    BtlAddrMsg(AddressPaneMsg),
    BtlAddrUpdated(AddressRange),
    IncludeToggle,
    PageSizeMsg(TextFieldMsg),
    PageSizeChanged(u64),
    WordAddressingToggle,
    TimeDataSendMsg(TextFieldMsg),
    TimeDataSendChanged(u32),
    TimeSendDoneMsg(TextFieldMsg),
    TimeSendDoneChanged(u32),
    TimeLeaveMsg(TextFieldMsg),
    TimeLeaveChanged(u32),
    TimeEraseMsg(TextFieldMsg),
    TimeEraseChanged(u32),
    HexSelectChanged(JsonValue),
    HeaderOffsetMsg(TextFieldMsg),
    HeaderOffsetChanged(u64),
}

pub struct FwPane {
    config: FwConfig,
    pub updated: Event<FwConfig>,
    pub remove: Event<()>,
    fw_id: TextField<u8>,
    btl_path: TextField<String>,
    app_path: TextField<String>,
    app_addr: AddressPane,
    btl_addr: AddressPane,
    include_id: String,
    page_size: TextField<u64>,
    word_addressing_id: String,
    header_offset: TextField<u64>,
    time_data_send: TextField<u32>,
    time_send_done: TextField<u32>,
    time_leave: TextField<u32>,
    time_erase: TextField<u32>,
}

impl Default for FwPane {
    fn default() -> Self {
        Self {
            config: Default::default(),
            updated: Default::default(),
            remove: Default::default(),
            fw_id: TextField::new(|x| u8::from_str_radix(x, 16).ok(),
                                  |x| format!("{:X}", x),
                                  1),
            btl_path: TextField::new(|x| Some(x.to_string()),
                                     |x| x.clone(),
                                     String::new()),
            app_path: TextField::new(|x| Some(x.to_string()),
                                     |x| x.clone(),
                                     String::new()),
            app_addr: Default::default(),
            btl_addr: Default::default(),
            include_id: format!("{}", Id::new().data()),
            word_addressing_id: format!("{}", Id::new().data()),
            header_offset: TextField::new(|x| u64::from_str_radix(x, 16).ok(),
                           |x| format!("{:X}", x),
                           0)
,
            time_data_send: Self::make_time_field(),
            time_send_done: Self::make_time_field(),
            time_leave: Self::make_time_field(),
            page_size: TextField::new(|x| u64::from_str_radix(x, 16).ok(),
                                      |x| format!("{:X}", x),
                                      2),
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

    fn make_time_field() -> TextField<u32> {
        TextField::new(|x| u32::from_str(x).ok(),
                       |x| x.to_string(),
                       0)
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
                self.emit(&ctx);
                Updated::no()
            }

            FwMsg::AppAddrMsg(msg) => self.app_addr.update(msg, ctx.map(FwMsg::AppAddrMsg)),
            FwMsg::AppAddrUpdated(range) => {
                self.config.app_address = range;
                self.emit(&ctx);
                Updated::no()
            }

            FwMsg::BtlAddrMsg(msg) => self.btl_addr.update(msg, ctx.map(FwMsg::BtlAddrMsg)),
            FwMsg::BtlAddrUpdated(range) => {
                self.config.btl_address = range;
                self.emit(&ctx);
                Updated::no()
            }
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
            FwMsg::PageSizeMsg(msg) => self.page_size.update(msg, &ctx),
            FwMsg::PageSizeChanged(page_size) => {
                self.config.device_config.page_size = page_size;
                self.emit(&ctx);
                Updated::no()
            }

            FwMsg::TimeDataSendMsg(msg) => self.time_data_send.update(msg, &ctx),
            FwMsg::TimeDataSendChanged(time) => {
                self.config.timings.data_send = time;
                self.emit(&ctx);
                Updated::no()
            }

            FwMsg::TimeSendDoneMsg(msg) => self.time_send_done.update(msg, &ctx),
            FwMsg::TimeSendDoneChanged(time) => {
                self.config.timings.data_send_done = time;
                self.emit(&ctx);
                Updated::no()
            }

            FwMsg::TimeLeaveMsg(msg) => self.time_leave.update(msg, &ctx),
            FwMsg::TimeLeaveChanged(time) => {
                self.config.timings.leave_btl = time;
                self.emit(&ctx);
                Updated::no()
            }

            FwMsg::TimeEraseMsg(msg) => self.time_erase.update(msg, &ctx),
            FwMsg::TimeEraseChanged(time) => {
                self.config.timings.erase_time = time;
                self.emit(&ctx);
                Updated::no()
            }

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
            FwMsg::HeaderOffsetMsg(msg) => self.header_offset.update(msg, &ctx),
            FwMsg::HeaderOffsetChanged(value) => {
                self.config.header_offset = value;
                self.emit(&ctx);
                Updated::no()
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
                    {self.fw_id.change_event().subscribe(FwMsg::FwIdChanged)}
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
                    {self.app_addr.changed.subscribe(FwMsg::AppAddrUpdated)}
                </div>
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-4">{"Btl Address"}</>
                    {self.btl_addr.render().map(FwMsg::BtlAddrMsg)}
                    {self.btl_addr.changed.subscribe(FwMsg::BtlAddrUpdated)}
                </div>
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-4">{"Page Size"}</>
                    {self.page_size.render().class("form-control flex-fill")
                        .attr("placeholder", "in hex").build().map(FwMsg::PageSizeMsg)}
                    {self.page_size.change_event().subscribe(FwMsg::PageSizeChanged)}
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
                    {self.header_offset.change_event().subscribe(FwMsg::HeaderOffsetChanged)}
                </div>

                // timings
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-6">{"Time between data send"}</>
                    {self.time_data_send.render().class("form-control flex-fill")
                        .attr("placeholder", "in ms").build().map(FwMsg::TimeDataSendMsg)}
                    {self.time_data_send.change_event().subscribe(FwMsg::TimeDataSendChanged)}
                </div>
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-6">{"Time after send done"}</>
                    {self.time_send_done.render().class("form-control flex-fill")
                        .attr("placeholder", "in ms").build().map(FwMsg::TimeSendDoneMsg)}
                    {self.time_send_done.change_event().subscribe(FwMsg::TimeSendDoneChanged)}
                </div>
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-6">{"Time to leave bootloader"}</>
                    {self.time_leave.render().class("form-control flex-fill")
                        .attr("placeholder", "in ms").build().map(FwMsg::TimeLeaveMsg)}
                    {self.time_leave.change_event().subscribe(FwMsg::TimeLeaveChanged)}
                </div>
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-6">{"Erase Time"}</>
                    {self.time_erase.render().class("form-control flex-fill")
                        .attr("placeholder", "in ms").build().map(FwMsg::TimeEraseMsg)}
                    {self.time_erase.change_event().subscribe(FwMsg::TimeEraseChanged)}
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
