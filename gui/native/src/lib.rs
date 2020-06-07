#![recursion_limit = "1024"]
#![allow(dead_code)]

use greenhorn::{Runtime, WebSocketPipe};
use neon::prelude::*;
use std::net::SocketAddr;
use std::str::FromStr;
use std::{panic, thread};

mod address_pane;
mod app;
mod fw_config;
mod text_field;

use crate::app::MainApp;
use backtrace::Backtrace;

fn run(mut cx: FunctionContext) -> JsResult<JsNumber> {
    panic::set_hook(Box::new(|info| {
        let bt = Backtrace::new();
        if let Some(loc) = info.payload().downcast_ref::<&str>() {
            println!("Panic occured: {}", loc)
        }
        println!("{:?}", bt);
    }));
    let addr = SocketAddr::from_str("127.0.0.1:0").unwrap();
    let pipe = WebSocketPipe::listen_to_addr(addr);
    let port = pipe.port();
    thread::spawn(|| {
        let app = MainApp::new();
        let (rt, _control) = Runtime::new(app, pipe);
        rt.run_blocking();
    });
    Ok(cx.number(port))
}

register_module!(mut cx, { cx.export_function("run", run) });
