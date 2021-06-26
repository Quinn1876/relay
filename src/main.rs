use std::env;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();

    let config;
    if args.len() > 1 {
        config = relay::tcp_server::Config::from_args(&args);
    } else {
        config = relay::tcp_server::Config::default();
    }


    relay::tcp_server::run(config).unwrap_or_else(|err| {
        println!("Error: {}", err);
    });
    Ok(())
}
