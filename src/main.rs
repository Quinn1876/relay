use std::env;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();

    // if cfg!(feature = "socketcan") {
    //     // let config;
    //     // if args.len() > 1 {
    //     //     config = crate::tcp_server::Config::from_args(&args);
    //     // } else {
    //     //     config = relay::tcp_server::Config::default();
    //     // }
    // }
    // let mut server = relay::tcp_server::Server::new(config);
    relay::tcp_server::run_threads().expect("Shutting down");

    // server.run_poll().unwrap_or_else(|err| {
    //     println!("Error: {:?}", err);
    // });
    // relay::tcp_server::run(config).unwrap_or_else(|err| {
    //     println!("Error: {}", err);
    // });
    Ok(())
}
