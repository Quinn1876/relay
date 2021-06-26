pub mod tcp_client {
    use std::io::prelude::*;
    use std::net::TcpStream;

    fn _client_example() -> Result<(), std::io::Error> {
        let mut stream = TcpStream::connect("0.0.0.0:34254")?;

        stream.write(&[1])?;
        stream.read(&mut [0; 128])?;
        Ok(())
    }

}

pub mod tcp_server {
    use std::net::{ TcpListener, TcpStream, IpAddr, Ipv4Addr, SocketAddr };
    use std::io::prelude::*;

    fn _handle_client(_stream: TcpStream) {
        ()
    }

    fn _bind_example() -> std::io::Result<()> {
        let listener = TcpListener::bind("0.0.0.0:34254")?;

        for stream in listener.incoming() {
            _handle_client(stream?);
        }
        Ok(())
    }

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
                    _ => (),
                }
                i -= 2;
            }
            config
        }
    }

    pub fn run<A: std::net::ToSocketAddrs>(config: Config<A>) -> std::io::Result<()> {
        let listener = TcpListener::bind(config.address)?;
        println!("Listening");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut request = Vec::<u8>::new();

                    let mut bytes_read = config.buffer_size;

                    while bytes_read == config.buffer_size {
                        let mut buffer = vec![0; config.buffer_size];
                        bytes_read = stream.read(&mut buffer)?;
                        request.append(&mut buffer);
                        println!("read in {} bytes", bytes_read);
                    }

                    let request = String::from_utf8(request).unwrap_or(String::from("@@Failed@@"));
                    println!("Request: \n{}", request);

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
}
