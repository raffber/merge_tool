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
            <div .main-app .container-fluid>
                <h1>{"Merge & Release Tool"}</>
                <div class="row align-items-center my-2">
                    <div class="custom-control custom-switch mx-1 col-auto">
                        <input type="checkbox" class="custom-control-input" id="auto-save-toggle" />
                        <label class="custom-control-label" for="auto-save-toggle">{"Auto Save"}</>
                    </>
                    <input type="text" class="col mx-1 form-control" placeholder="Path to config file..." />
                    <button type="button" class="btn btn-secondary mx-1 col-auto">{"Open"}</>
                </>
                <div class="row align-items-center my-2">
                    <span class="col-3">{"Product ID"}</>
                    <input type="text" class="col-3 form-control" placeholder="e.g. 0xABCD"/>
                    <span class="col-3">{"Product Name"}</>
                    <input type="text" class="col-3 form-control" placeholder="e.g. Nimbus2000"/>
                </>
                <div class="row align-items-center my-2">
                    <span class="col-3">{"State Transition Time"}</>
                    <input type="text" class="col-3 form-control" placeholder="in ms"/>
                    <div class="col-6 form-check">
                        <input type="checkbox" class="form-check-input" id="use-backdoor" />
                        <label class="form-check-label" for="use-backdoor">
                        {"Use Backdoor"}
                        </>
                    </>                    
                </>
                <div #main-button-row>                    
                    <button type="button" class="btn btn-secondary mx-1">{"Merge"}</>
                    <button type="button" class="btn btn-secondary mx-1">{"Release"}</>
                    <button type="button" class="btn btn-primary mx-1">{"Generate Script"}</>
                </>
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