use status::Bar;

mod status;

fn main() {
    env_logger::init();

    let (mut state, mut event_queue) = Bar::new();

    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();
        if state.req_exit() {
            std::process::exit(0);
        }
    }
}
