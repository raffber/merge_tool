use crate::text_field::{TextField, TextFieldMsg};
use greenhorn::html;
use greenhorn::prelude::*;
use merge_tool::config::AddressRange;

// TODO: deny end < begin

#[derive(Debug)]
pub enum AddressPaneMsg {
    BeginMsg(TextFieldMsg),
    EndMsg(TextFieldMsg),
}

pub struct AddressPane {
    pub data: AddressRange,
    pub changed: Event<AddressRange>,
    begin_field: TextField<u64>,
    end_field: TextField<u64>,
}

impl Default for AddressPane {
    fn default() -> Self {
        AddressPane::new(&Default::default())
    }
}

impl AddressPane {
    pub fn new(range: &AddressRange) -> Self {
        Self {
            data: range.clone(),
            changed: Default::default(),
            begin_field: Self::make_text_field().with_value(range.begin),
            end_field: Self::make_text_field().with_value(range.end),
        }
    }

    pub fn set(&mut self, range: &AddressRange) {
        self.begin_field.set(range.begin);
        self.end_field.set(range.end);
    }

    fn make_text_field() -> TextField<u64> {
        TextField::new(|x| u64::from_str_radix(x, 16).ok(), |x| format!("{:X}", x))
    }

    pub fn update(&mut self, msg: AddressPaneMsg, data: &mut AddressRange) -> (bool, Updated) {
        match msg {
            AddressPaneMsg::BeginMsg(msg) => self.begin_field.update(&mut data.begin, msg),
            AddressPaneMsg::EndMsg(msg) => self.end_field.update(&mut data.end, msg),
        }
    }
}

impl Render for AddressPane {
    type Message = AddressPaneMsg;

    fn render(&self) -> Node<Self::Message> {
        html!(
            <div class="d-flex flex-row align-items-center flex-fill">
                <div class="input-group flex-fill">
                    <div class="input-group-prepend"> <span class="input-group-text">{"0x"}</span> </>
                    {self.begin_field.render().class("form-control")
                        .attr("placeholder", "in hex").build().map(AddressPaneMsg::BeginMsg)}
                </>
                <span class="mx-2">{"to"}</>
                <div class="input-group flex-fill">
                    <div class="input-group-prepend"> <span class="input-group-text">{"0x"}</span> </>
                    {self.end_field.render() .class("form-control")
                        .attr("placeholder", "in hex").build().map(AddressPaneMsg::EndMsg)}
                </>
            </>
        )
        .into()
    }
}
