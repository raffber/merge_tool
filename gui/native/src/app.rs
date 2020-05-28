use greenhorn::prelude::*;

pub enum MainMsg {
    SayHello,
    SayGoodbye,    
}

pub struct MainApp {
    msg: String,
}

impl MainApp {
    pub fn new() -> Self {
        Self {
            msg: "something".into(),
        }
    }
}

impl Render for MainApp {
    type Message = MainMsg;

    fn render(&self) -> Node<Self::Message> {
        use greenhorn::html;

        html!(
            <div>
                <h1>{format!("Say {}!", self.msg)}</>
                <input type="button" @click={|_| MainMsg::SayHello} value="Hello" />
                <input type="button" @click={|_| MainMsg::SayGoodbye} value="Goodbye" />
            </>
        ).into()
    }
}

impl App for MainApp {
    fn update(&mut self, msg: Self::Message, _ctx: Context<Self::Message>) -> Updated {
        match msg {
            MainMsg::SayHello => self.msg = "hello".into(),
            MainMsg::SayGoodbye => self.msg = "goodbye".into()
        }
        Updated::yes()
    }    
}