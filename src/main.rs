mod constants;
mod discriminate;
mod state;
mod ui;
mod utils;

use ggez::{ContextBuilder, conf, event};
use std::{env, path};

use crate::state::State;

fn main() {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let mut c = conf::Conf::new();
    c.window_setup = c.window_setup.vsync(false).samples(conf::NumSamples::Four);
    let (mut ctx, event_loop) = ContextBuilder::new("hello_ggez", "test")
        .add_resource_path(resource_dir)
        .default_conf(c)
        .build()
        .unwrap();

    let state = State::new(&mut ctx).unwrap();

    event::run(ctx, event_loop, state).unwrap();
}
