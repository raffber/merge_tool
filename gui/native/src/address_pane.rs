use greenhorn::prelude::*;
use greenhorn::html;
use crate::text_field::{TextField, TextFieldMsg};
use merge_tool::config::AddressRange;

// TODO: deny end < begin

#[derive(Debug)]
pub enum AddressPaneMsg {
    BeginMsg(TextFieldMsg),
    EndMsg(TextFieldMsg),
    EndUpdated(u64),
    BeginUpdated(u64),
}

pub struct AddressPane {
    pub data: AddressRange,
    pub changed: Event<AddressRange>,
    begin_field: TextField<u64>,
    end_field: TextField<u64>,
}

impl Default for AddressPane {
    fn default() -> Self {
        AddressPane::new(Default::default())
    }
}

impl AddressPane {
    pub fn new(range: AddressRange) -> Self {
        Self {
            data: range.clone(),
            changed: Default::default(),
            begin_field: Self::make_text_field(range.begin),
            end_field: Self::make_text_field(range.end),
        }
    }

    fn make_text_field(initial: u64) -> TextField<u64> {
        TextField::new(
            |x| u64::from_str_radix(x, 16).ok(),
            |x| format!("{:X}", x),
            initial)
    }
}

impl App for AddressPane {
    fn update(&mut self, msg: Self::Message, ctx: Context<Self::Message>) -> Updated {
        match msg {
            AddressPaneMsg::BeginMsg(msg) => self.begin_field.update(msg, &ctx),
            AddressPaneMsg::EndMsg(msg) => self.end_field.update(msg, &ctx),
            AddressPaneMsg::EndUpdated(value) => {
                self.data.end = value;
                ctx.emit(&self.changed, self.data.clone());
                Updated::no()
            },
            AddressPaneMsg::BeginUpdated(value) => {
                self.data.begin = value;
                ctx.emit(&self.changed, self.data.clone());
                Updated::no()
            },
        }
    }
}

impl Render for AddressPane {
    type Message = AddressPaneMsg;

    fn render(&self) -> Node<Self::Message> {
        html!(
            <div class="d-flex flex-row align-items-center flex-fill">
                {self.begin_field.render().class("form-control flex-fill")
                    .attr("placeholder", "in hex").build().map(AddressPaneMsg::BeginMsg)}
                <span class="mx-2">{"to"}</>
                {self.end_field.render() .class("form-control flex-fill")
                    .attr("placeholder", "in hex").build().map(AddressPaneMsg::EndMsg)}

                {self.begin_field.change_event().subscribe(AddressPaneMsg::BeginUpdated)}
                {self.end_field.change_event().subscribe(AddressPaneMsg::EndUpdated)}
            </>
        ).into()
    }
}

