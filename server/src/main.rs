use log::LevelFilter;

mod r#async;
// mod sync;

fn main() {
    env_logger::builder()
        .filter_module("mc", LevelFilter::Trace)
        .filter_module("server", LevelFilter::Debug)
        .filter_level(LevelFilter::Info)
        .init();

    r#async::main();
}
