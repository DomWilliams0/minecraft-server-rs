use log::LevelFilter;

mod server;

fn main() {
    env_logger::builder()
        .filter_module("mc", LevelFilter::Trace)
        .filter_module("server", LevelFilter::Debug)
        .filter_level(LevelFilter::Info)
        .init();

    server::main();
}
