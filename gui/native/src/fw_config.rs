use greenhorn::prelude::*;
use greenhorn::html;
use merge_tool::config::FwConfig;
use crate::text_field::TextField;

#[derive(Debug)]
pub enum FwMsg {
    RemoveClicked,
    UpdateConfig(FwConfig)
}

#[derive(Default)]
pub struct FwPane {
    config: FwConfig,
    pub updated: Event<FwConfig>,
    pub remove: Event<()>,
    fw_id: TextField<u16>,
    btl_path: TextField<String>,
    app_path: TextField<String>,

}

impl FwPane {
    pub fn new() -> Self {
        Default::default()
    }
}

impl App for FwPane {
    fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated {
        match msg {
            FwMsg::RemoveClicked => {
                ctx.emit(&self.remove, ());
            },

            FwMsg::UpdateConfig(config) => {
                self.config = config;
            }
        }
        Updated::yes()
    }
}

impl Render for FwPane {
    type Message = FwMsg;

    fn render(&self) -> Node<Self::Message> {
        html!( <div #fw-config >
                // row with product ID + Product name
                <div class="d-flex flex-row align-items-center my-2">
                    <span class="col-6">{"Firmware ID"}</>
                    {self.fw_id.render().build().map(Msg::ProductIdMsg)}
                </>
                    {self.product_id.change_event().subscribe(Msg::ProductIdChanged)}
                    <span class="col-3">{"Product Name"}</>
                    {self.product_name.render().build().map(Msg::ProductNameMsg)}
                    {self.product_name.change_event().subscribe(Msg::ProductNameChanged)}
                </>

        </> ).into()
    }
}
