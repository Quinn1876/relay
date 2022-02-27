use std::env;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();

    // if cfg!(feature = "socketcan") {
    let config;
    if args.len() > 1 {
        config = relay::config::Config::from_args(&args);
    } else {
        config = relay::config::Config::default();
    }
    // }
    // let mut server = relay::tcp_server::Server::new(config);
    relay::run_threads::run_threads(config).expect("Shutting down");
    Ok(())
}
