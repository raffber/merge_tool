use greenhorn::prelude::*;

#[derive(Debug)]
pub enum SelectionBoxMsg {
    SelectedIndexChanged(JsonValue),
}


pub struct SelectionBox {
    items: Vec<String>,
    version: Id,
    selected_index: u32,
}

impl SelectionBox {
    pub fn new(items: Vec<String>, selected_index: u32) -> Self {
        Self {
            items,
            version: Default::default(),
            selected_index,
        }
    }

    pub fn set(&mut self, idx: u32) {
        self.selected_index = idx;
        self.version = Id::new();
    }

    pub fn update(&mut self, msg: SelectionBoxMsg) -> u32 {
        match msg {
            SelectionBoxMsg::SelectedIndexChanged(value) => {
                let idx: u32 = serde_json::from_value(value).unwrap();
                if idx >= self.items.len() as u32 {
                    panic!();
                }
                self.selected_index = idx;
            },
        }
        self.selected_index
    }

    pub fn render(&self) -> ElementBuilder<SelectionBoxMsg> {
        let render_fun = "{
            let rendered_version = event.target.getAttribute('__rendered_version');
            let value_version = event.target.getAttribute('__value_version');
            if (rendered_version != value_version) {
                event.target.selectedIndex = event.target.getAttribute('__selected_index');
                event.target.setAttribute('__rendered_version', value_version);
            }
        }";

        Node::html().elem("select")
            .js_event("change", "app.send(event.target, event.target.selectedIndex)")
            .js_event("render", render_fun)
            .rpc(SelectionBoxMsg::SelectedIndexChanged)
            .attr("__value_version", self.version)
            .attr("__selected_index", self.selected_index)
            .add( self.items.iter().map(|x| Node::html().elem("option").text(x).build() ) )
    }
}
