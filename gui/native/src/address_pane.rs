use greenhorn::prelude::*;
use greenhorn::html;
use merge_tool::config::AddressRange;
use crate::lean_field::{LeanMsg, LeanField};

// TODO: deny end < begin

#[derive(Debug)]
pub enum AddressPaneMsg {
    BeginMsg(LeanMsg),
    EndMsg(LeanMsg),
}

pub struct AddressPane {
    pub data: AddressRange,
    pub changed: Event<AddressRange>,
    begin_field: LeanField<u64>,
    end_field: LeanField<u64>,
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

    fn make_text_field() -> LeanField<u64> {
        LeanField::new(
            |x| u64::from_str_radix(x, 16).ok(),
            |x| format!("{:X}", x))
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
                {self.begin_field.render().class("form-control flex-fill")
                    .attr("placeholder", "in hex").build().map(AddressPaneMsg::BeginMsg)}
                <span class="mx-2">{"to"}</>
                {self.end_field.render() .class("form-control flex-fill")
                    .attr("placeholder", "in hex").build().map(AddressPaneMsg::EndMsg)}
            </>
        ).into()
    }
}

