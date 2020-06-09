use crate::address_pane::{AddressPane, AddressPaneMsg};
use crate::selection_box::{SelectionBox, SelectionBoxMsg};
use crate::text_field::{TextField, TextFieldMsg};
use greenhorn::components::checkbox;
use greenhorn::dialog::{FileFilter, FileOpenDialog, FileOpenMsg};
use greenhorn::html;
use greenhorn::prelude::*;
use merge_tool::config::{FwConfig, HexFileFormat, Config};
use std::str::FromStr;
use std::path::{Path, PathBuf};

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
    AppAddrMsg(AddressPaneMsg),
    BtlAddrMsg(AddressPaneMsg),
    IncludeToggle,
    PageSizeMsg(TextFieldMsg),
    WordAddressingToggle,
    TimeDataSendMsg(TextFieldMsg),
    TimeSendDoneMsg(TextFieldMsg),
    TimeLeaveMsg(TextFieldMsg),
    TimeEraseMsg(TextFieldMsg),
    HexSelectMsg(SelectionBoxMsg),
    HeaderOffsetMsg(TextFieldMsg),
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
    hex_file_type: SelectionBox,
    config_path: Option<PathBuf>,
}

impl Default for FwPane {
    fn default() -> Self {
        let hex_file_types = vec!["Intel Hex *.hex".to_string(), "S-Record *.s37".to_string()];
        Self {
            config: Default::default(),
            updated: Default::default(),
            remove: Default::default(),
            fw_id: TextField::new(|x| u8::from_str_radix(x, 16).ok(), |x| format!("{:X}", x)),
            btl_path: TextField::new(|x| Some(x.to_string()), |x| x.clone()),
            app_path: TextField::new(|x| Some(x.to_string()), |x| x.clone()),
            app_addr: Default::default(),
            btl_addr: Default::default(),
            include_id: format!("{}", Id::new().data()),
            word_addressing_id: format!("{}", Id::new().data()),
            header_offset: TextField::new(
                |x| u64::from_str_radix(x, 16).ok(),
                |x| format!("{:X}", x),
            ),
            time_data_send: Self::make_time_field(),
            time_send_done: Self::make_time_field(),
            time_leave: Self::make_time_field(),
            page_size: TextField::new(|x| u64::from_str_radix(x, 16).ok(), |x| format!("{:X}", x)),
            time_erase: Self::make_time_field(),
            hex_file_type: SelectionBox::new(hex_file_types, 0),
            config_path: None
        }
    }
}

impl FwPane {
    pub fn new() -> Self {
        let mut ret: FwPane = Default::default();
        ret.config.device_config.page_size = 2;
        ret
    }

    pub fn set_config_path(&mut self, config_path: &Path) {
        self.config_path = Some(config_path.to_path_buf());
    }

    fn make_time_field() -> TextField<u32> {
        TextField::new(|x| u32::from_str(x).ok(), |x| x.to_string())
    }

    pub fn with_config(config: &FwConfig) -> Self {
        let mut ret = Self::new();
        ret.apply(config);
        ret
    }

    pub fn apply(&mut self, config: &FwConfig) {
        self.config = config.clone();
        self.fw_id.set(config.fw_id);
        self.app_path.set(config.app_path.clone());
        self.btl_path.set(config.btl_path.clone());
        self.header_offset.set(config.header_offset);
        self.page_size.set(config.device_config.page_size);
        self.app_addr.set(&config.app_address);
        self.btl_addr.set(&config.btl_address);
        self.time_data_send.set(config.timings.data_send);
        self.time_send_done.set(config.timings.data_send_done);
        self.time_erase.set(config.timings.erase_time);
        self.time_leave.set(config.timings.leave_btl);
        match config.hex_file_format {
            HexFileFormat::IntelHex => {
                self.hex_file_type.set(0);
            }
            HexFileFormat::SRecord => {
                self.hex_file_type.set(1);
            }
        }
    }

    fn open_hex_file(&self) -> FileOpenDialog {
        FileOpenDialog::new("Open hex file...", "~")
            .with_filter(FileFilter::new("hex files").push("s37").push("hex"))
    }

    fn make_path_relative(&self, path: &str) -> String {
        if let Some(config_path) = &self.config_path {
            if let Ok(config_dir) = Config::get_config_dir(&config_path) {
                let path = Path::new(path);
                if let Some(path) = pathdiff::diff_paths(path, config_dir) {
                    if let Some(path) = path.to_str() {
                        return path.to_string()
                    }
                }
            }
        }
        path.to_string()
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
            }

            FwMsg::UpdateConfig(config) => {
                self.config = config;
                Updated::yes()
            }
            FwMsg::FwIdMsg(msg) => {
                let ret = self.fw_id.update(&mut self.config.fw_id, msg);
                self.prop(&ctx, ret)
            }
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
            }
            FwMsg::BtlPathMsg(msg) => {
                let ret = self.btl_path.update(&mut self.config.btl_path, msg);
                self.prop(&ctx, ret)
            }

            FwMsg::AppAddrMsg(msg) => {
                let (changed, ret) = self.app_addr.update(msg, &mut self.config.app_address);
                if changed {
                    self.emit(&ctx);
                }
                ret
            }

            FwMsg::BtlAddrMsg(msg) => {
                let (changed, ret) = self.btl_addr.update(msg, &mut self.config.btl_address);
                if changed {
                    self.emit(&ctx);
                }
                ret
            }

            FwMsg::IncludeToggle => {
                self.config.include_in_script = !self.config.include_in_script;
                self.emit(&ctx);
                Updated::yes()
            }
            FwMsg::WordAddressingToggle => {
                self.config.device_config.word_addressing =
                    !self.config.device_config.word_addressing;
                self.emit(&ctx);
                Updated::yes()
            }
            FwMsg::PageSizeMsg(msg) => {
                let ret = self
                    .page_size
                    .update(&mut self.config.device_config.page_size, msg);
                self.prop(&ctx, ret)
            }

            FwMsg::TimeDataSendMsg(msg) => {
                let ret = self
                    .time_data_send
                    .update(&mut self.config.timings.data_send, msg);
                self.prop(&ctx, ret)
            }

            FwMsg::TimeSendDoneMsg(msg) => {
                let ret = self
                    .time_send_done
                    .update(&mut self.config.timings.data_send_done, msg);
                self.prop(&ctx, ret)
            }

            FwMsg::TimeLeaveMsg(msg) => {
                let ret = self
                    .time_leave
                    .update(&mut self.config.timings.leave_btl, msg);
                self.prop(&ctx, ret)
            }

            FwMsg::TimeEraseMsg(msg) => {
                let ret = self
                    .time_erase
                    .update(&mut self.config.timings.erase_time, msg);
                self.prop(&ctx, ret)
            }

            FwMsg::HexSelectMsg(msg) => {
                let kind = self.hex_file_type.update(msg);
                self.config.hex_file_format = match kind {
                    0 => HexFileFormat::IntelHex,
                    1 => HexFileFormat::SRecord,
                    _ => panic!(),
                };
                self.emit(&ctx);
                Updated::no()
            }

            FwMsg::HeaderOffsetMsg(msg) => {
                let ret = self
                    .header_offset
                    .update(&mut self.config.header_offset, msg);
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
                    <div class="input-group flex-fill">
                        <div class="input-group-prepend"> <span class="input-group-text">{"0x"}</span> </>
                        {self.fw_id.render().class("form-control").build().map(FwMsg::FwIdMsg)}
                    </>
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

                    <div class="input-group flex-fill">
                        <div class="input-group-prepend"> <span class="input-group-text">{"0x"}</span> </>
                        {self.page_size.render().class("form-control")
                            .attr("placeholder", "in hex").build().map(FwMsg::PageSizeMsg)}
                    </>
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
                    {self.hex_file_type.render().class("form-control").build().map(FwMsg::HexSelectMsg)}
                </>
                <div class="d-flex flex-row align-items-center my-1">
                    <span class="col-4">{"Header Offset"}</>
                    <div class="input-group flex-fill">
                        <div class="input-group-prepend"> <span class="input-group-text">{"0x"}</span> </>
                        {self.header_offset.render().class("form-control flex-fill")
                            .attr("placeholder", "in hex").build().map(FwMsg::HeaderOffsetMsg)}
                    </>
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
