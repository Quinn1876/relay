mod stream_utils;
mod requests;

pub mod tcp_server {
    use super::requests;
    #[cfg(test)]
    mod test {
        use super::*;
        use std::net::{SocketAddr, IpAddr, Ipv4Addr};
        #[test]
        fn config_from_args_address() {
            let args = vec!["test program", "-a", "100.20.20.10:9090"];
            let args: Vec<String> = args.iter().map(|&arg| String::from(arg)).collect();
            let config_dut = Config::from_args(&args);
            let expected_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(100, 20, 20, 10)), 9090);
            assert_eq!(config_dut.address.ip(), expected_address.ip());
            assert_eq!(config_dut.address.port(), expected_address.port());
        }

        #[test]
        fn config_from_args_buffer_size() {
            let args = vec!["test program", "-b", "512"];
            let args: Vec<String> = args.iter().map(|&arg| String::from(arg)).collect();

            let config_dut = Config::from_args(&args);
            let expected_size: usize = 512;

            assert_eq!(config_dut.buffer_size, expected_size);
        }

        #[test]
        fn config_from_args_buffer_size_and_address() {
            let args = vec!["test program", "-b", "1024", "-a", "250.230.210.120:1000"];
            let args: Vec<String> = args.iter().map(|arg| String::from(*arg)).collect();

            let config_dut = Config::from_args(&args);

            let expected_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(250, 230, 210, 120)), 1000);
            let expected_size: usize = 1024;

            assert_eq!(config_dut.address.ip(), expected_address.ip());
            assert_eq!(config_dut.address.port(), expected_address.port());
            assert_eq!(config_dut.buffer_size, expected_size);
        }
    }


    use std::net::{ TcpListener, TcpStream, IpAddr, Ipv4Addr, SocketAddr, UdpSocket };
    use std::io::prelude::*;

    pub struct Config<A: std::net::ToSocketAddrs> {
        address: A,
        buffer_size: usize,
    }

    impl<A: std::net::ToSocketAddrs> Config<A> {
        pub fn new(address: A, buffer_size: usize) -> Config<A> {
            Config {
                address,
                buffer_size,
            }
        }
    }

    impl Config<SocketAddr> {
        pub fn default() -> Config<SocketAddr> {
            Config {
                address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
                buffer_size: 256,
            }
        }

        /**
         * @brief from_args
         * This builds a Config Item from a vector of command line arguments
         *
         * If the args vector is malformed, the function will panic and exit
         * TODO add proper error handling
         *
         * Currently Accepted arguments:
         * -a hostIpv4:port
         * -b buffer_size
         */
        pub fn from_args(args: &Vec<String>) -> Config<SocketAddr> {
            if args.len() % 2 == 0 {
                panic!("invalid arguments");
            }
            let mut i = args.len() - 1;
            let mut config = Config::default();

            while i > 1 {
                let param = &args[i];
                let param_type: &str = &args[i-1];

                match param_type {
                    "-a" => {
                        let host_and_port: Vec<&str> = param.split(':').collect();
                        if host_and_port.len() != 2 {
                            panic!("Invalid address Argument, expected form -a <host>:<port>");
                        }
                        let host = host_and_port[0];
                        let port = host_and_port[1].parse::<u16>().unwrap();
                        let host: Vec<&str> = host.split('.').collect();

                        if host.len() != 4 {
                            panic!("Invalid host, expected form ##.##.##.##");
                        }

                        let host: Vec<u8> = host.iter().map(|val| val.parse::<u8>().unwrap()).collect();

                        config.address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(host[0], host[1], host[2], host[3])), port);
                    },
                    "-b" => {
                        let size = param.parse::<usize>().unwrap();

                        config.buffer_size = size;
                    }
                    _ => (),
                }
                i -= 2; // read arguments in pairs
            }
            config
        }
    }

    enum RequestTypes {
        Handshake,
        Unknown
    }

    pub fn run<A: std::net::ToSocketAddrs>(config: Config<A>) -> std::io::Result<()> {
        let listener = TcpListener::bind(config.address)?;
        println!("Listening on {}", listener.local_addr().ok().unwrap());

        let mut request_parser: requests::RequestParser::<&RequestTypes> = requests::RequestParser::new();
        request_parser.insert("HANDSHAKE\r\n", &RequestTypes::Handshake);
        request_parser.insert("@@Failed@@\r\n", &RequestTypes::Unknown);
        let request_parser = request_parser;

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let incoming_socket = stream.peer_addr().unwrap();
                    println!("{} Connected", incoming_socket);
                    let incoming_addr = incoming_socket.ip();

                    let request = super::stream_utils::read_all(&mut stream, config.buffer_size)?;
                    match request_parser.strip_line_and_get_value(request.as_slice()) {
                        requests::RequestParserResult::Success((&value, request)) => {
                            match value {
                                RequestTypes::Handshake => {
                                    println!("HandShake received");
                                    handle_handshake(request, &mut stream).unwrap();
                                },
                                _ => ()
                            }
                        },
                        requests::RequestParserResult::InvalidRequest => {
                            println!("Invalid Request Recieved");
                        },
                        _ => ()
                    };

                    let response = "aaaa\r\n\r\n";
                    stream.write(response.as_bytes())?;
                    ()
                }
                Err(err) => {
                    println!("Error Connecting to stream: {}", err);
                }
            }
        }
        Ok(())
    }

    fn handle_handshake(request: &[u8], stream: &mut TcpStream) -> std::io::Result<()> {
        let desktop = "DESKTOP\r\n";
        let request = &request[0..desktop.len()];
        let request = String::from_utf8(request.to_vec()).unwrap();
        println!("{} {} {}", request, desktop, request == desktop);

        if request == desktop {
            let mut addr = stream.local_addr()?;
            addr.set_port(8888);
            let mut udp_socket = UdpSocket::bind(addr)?;
            println!("Bound to udpSocket {}", addr);
            stream.write(b"8888")?;

            let mut buffer = [0u8; 256];

            let (amount, src) = udp_socket.recv_from(&mut buffer)?;

            println!("UDP Packet: {}", String::from_utf8(buffer.to_vec()).unwrap());

        }
        Ok(())
    }
}
