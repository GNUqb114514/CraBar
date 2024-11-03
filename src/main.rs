use clap::Parser;
use status::Bar;

mod cli;
mod consts;
mod paint;
mod parse;
mod status;

fn main() {
    env_logger::init();

    let config = cli::Config::parse();
    let (mut state, mut event_queue) = Bar::new(config);
    let data = state.data();
    let condvar = state.condvar();

    let io_thread = std::thread::spawn(move || {
        loop {
            let mut input = String::new();
            log::info!("Waiting input...");
            let stdin = std::io::stdin();
            match stdin.read_line(&mut input) {
                Ok(n) => {
                    if n == 0 {
                        log::info!("n == 0; exiting");
                        break;
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                    log::info!("Broken pipe; exit normally");
                    break;
                }
                Err(ref e) => {
                    log::error!("Cannot get new input: {}", e.kind());
                    break;
                }
            }
            input.pop(); // Remove trailing space

            // Updata mutex
            let mut mutex = data.lock().unwrap();
            mutex.0 = input;
            mutex.1 = true;
            condvar.notify_one();
        }
    });

    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();
        if state.req_exit() || io_thread.is_finished() {
            std::process::exit(0);
        }
    }
}
