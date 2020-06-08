use greenhorn::prelude::*;

#[derive(Debug)]
pub enum TextFieldMsg {
    KeyUp(DomEvent),
}

pub struct TextField<T: 'static + Clone + Default> {
    value: T,
    version: Id,
    valid: bool,
    validator: Box<dyn Send + Fn(&str) -> Option<T>>,
    to_string: Box<dyn Send + Fn(&T) -> String>,
}

impl<T: 'static + Clone + Default> TextField<T> {
    pub fn new<F: 'static + Send + Fn(&str) -> Option<T>, S: 'static + Send + Fn(&T) -> String>(
        fun: F,
        to_string: S,
    ) -> Self {
        let inital = Default::default();
        let text = to_string(&inital);
        let valid = fun(&text).is_some();
        Self {
            value: inital,
            version: Id::new(),
            valid,
            validator: Box::new(fun),
            to_string: Box::new(to_string),
        }
    }

    pub fn with_value(mut self, value: T) -> Self {
        self.value = value;
        self
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
        self.version = Id::new();
        let text = (*self.to_string)(&self.value);
        self.valid = (*self.validator)(&text).is_some();
    }

    pub fn update(&mut self, value: &mut T, msg: TextFieldMsg) -> (bool, Updated) {
        match msg {
            TextFieldMsg::KeyUp(evt) => {
                let text = evt.target_value().get_text().unwrap();
                let old_valid = self.valid;
                if let Some(v) = (*self.validator)(&text) {
                    *value = v.clone();
                    self.value = v.clone();
                    self.valid = true;
                } else {
                    self.valid = false;
                }
                if old_valid != self.valid {
                    (self.valid, Updated::yes())
                } else {
                    (self.valid, Updated::no())
                }
            }
        }
    }

    pub fn render(&self) -> ElementBuilder<TextFieldMsg> {
        let render_fun = "{
            let rendered_version = event.target.getAttribute('__rendered_version');
            let value_version = event.target.getAttribute('__value_version');
            if (rendered_version != value_version) {
                event.target.value = event.target.getAttribute('value');
                event.target.setAttribute('__rendered_version', value_version);
            }
        }";
        let text = (*self.to_string)(&self.value);
        let mut ret = Node::html()
            .elem("input")
            .attr("type", "text")
            .attr("__value_version", self.version.data())
            .on("keyup", TextFieldMsg::KeyUp)
            .attr("value", text)
            .js_event("render", render_fun);
        if !self.valid {
            ret = ret.class("is-invalid");
        }
        ret
    }
}
