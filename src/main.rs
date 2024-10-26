use clap::Parser;
use status::Bar;

mod cli;
mod parse;
mod status;
#[cfg(test)]
mod test;
mod paint;

fn main() {
    env_logger::init();

    let config = cli::Config::parse();
    let (mut state, mut event_queue) = Bar::new(config);

    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();
        if state.req_exit() {
            std::process::exit(0);
        }
    }
}
