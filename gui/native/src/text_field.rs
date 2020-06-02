use greenhorn::prelude::*;

pub struct TextField<T: 'static + Clone> {
    value: T,
    version: u32,
    valid: bool,
    change: Event<T>,
    validator: Box<dyn Fn(&str) -> Option<T>>,
    to_string: Box<dyn Fn(&T) -> String>,
    class: String,
    placeholder: String,
}

#[derive(Debug)]
pub enum TextFieldMsg {
    KeyUp(DomEvent),
}

impl<T: 'static + Clone> TextField<T> {
    pub fn new<F: 'static + Fn(&str) -> Option<T>, S: 'static + Fn(&T) -> String>
            (fun: F, to_string: S, inital: T) -> Self {
        let text = to_string(&inital);
        let valid = fun(&text).is_some();
        Self {
            value: inital,
            version: 0,
            valid,
            change: Default::default(),
            validator: Box::new(fun),
            to_string: Box::new(to_string),
            class: "".to_string(),
            placeholder: "".to_string(),
        }
    }

    pub fn class<S: Into<String>>(mut self, value: S) -> Self {
        self.class = value.into();
        self
    }

    pub fn placeholder<S: Into<String>>(mut self, value: S) -> Self {
        self.placeholder = value.into();
        self
    }

    pub fn change_event(&self) -> &Event<T> {
        &self.change
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
        self.version += 1;
    }

    pub fn get(&self) -> &T { &self.value }

    pub fn update<M: 'static + Send>(&mut self, msg: TextFieldMsg, ctx: &Context<M>) -> Updated {
        match msg {
            TextFieldMsg::KeyUp(evt) => {
                let text = evt.target_value().get_text().unwrap();
                let old_valid = self.valid;
                if let Some(value) = (*self.validator)(&text) {
                    self.value = value;
                    ctx.emit(&self.change, self.value.clone());
                    self.valid = true;
                } else {
                    self.valid = false;
                }
                if old_valid != self.valid {
                    Updated::yes()
                } else {
                    Updated::no()
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
            .attr("__value_version", self.version)
            .on("keyup", TextFieldMsg::KeyUp)
            .attr("value", text)
            .class(self.class.clone())
            .attr("placeholder", self.placeholder.clone())
            .js_event("render", render_fun);
        if !self.valid {
            ret = ret.class("is-invalid");
        }
        ret
    }
}
