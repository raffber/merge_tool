use greenhorn::prelude::*;
use greenhorn::html;
use merge_tool::config::FwConfig;

#[derive(Debug)]
pub enum FwMsg {
    RemoveClicked,
    UpdateConfig(FwConfig)
}

#[derive(Default)]
pub struct FwPane {
    config: FwConfig,
    pub updated: Event<FwConfig>,
    pub remove: Event<()>
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
        html!( <div #fw-config>{"Foobar uaahh!!!!"}</> ).into()
    }
}
