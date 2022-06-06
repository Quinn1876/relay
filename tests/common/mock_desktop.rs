use json::object;
use std::net::{
  TcpStream,
  UdpSocket,
  ToSocketAddrs,
  SocketAddr,
  IpAddr,
  Ipv4Addr
};
use relay::pod_states::PodState;
use relay::pod_data::PodData;
use relay::utils::device_watchdog::get_now;
use std::thread;
use std::sync::mpsc::{
  Sender,
  Receiver,
  channel
};
use std::error::Error;
use std::io::{
  Read,
  Write
};

pub struct MockDesktop<T: ToSocketAddrs> {
  relay_board_tcp_addr: T,
  thread_handle: Option<thread::JoinHandle<()>>,
  main_thread_sender: Sender<MockDesktopMessage>,
  main_thread_receiver: Receiver<MockDesktopMessage>
}

enum MockDesktopMessage {
  PodDataReceived(PodData)
}

impl<T: ToSocketAddrs> MockDesktop<T> {
  pub fn new(tcp_ip: T)
  -> MockDesktop<T> {
    let (main_thread_sender, main_thread_receiver) = channel::<MockDesktopMessage>();
    MockDesktop {
      relay_board_tcp_addr: tcp_ip,
      thread_handle: None,
      main_thread_sender,
      main_thread_receiver
    }
  }

  pub fn Connect(&mut self) -> Result<(), Box<dyn Error>> {
    /* Send Connect Request */
    let tcp_stream = TcpStream::connect(self.relay_board_tcp_addr).unwrap();
    tcp_stream.write("CONNECT\r\n".as_bytes())?;

    /* Receive Response  */
    let mut response = String::default();
    tcp_stream.read_to_string(&mut response)?;

    /* Parse Response */
    let mut response: Vec<&str> = response.split(" ").collect();
    let bind_port = response.get(1).unwrap().parse::<u16>()?;
    let relay_ip = tcp_stream.peer_addr().unwrap().ip();
    let relay_board_port = response.get(2).unwrap().parse::<u16>()?;
    let relay_board_addr = SocketAddr::new(relay_ip, relay_board_port);

    /* Setup UDP Socket for communication with relay board */
    let udp_socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::from([0,0,0,0])), bind_port))?;
    udp_socket.connect(relay_board_addr).unwrap();

    /* Set Up IPC Pipelines */

    self.thread_handle = Some(std::thread::Builder::new().name("UDP Thread".to_string()).spawn(move || {
      let mut timestamp = get_now();
      let mut recv_buf: [u8; 1024] = [0; 1024];
      // udp_socket.
      loop {
        udp_socket.send(&object!{ "requested_state": PodState::LowVoltage.to_byte(), "most_recent_timestamp": timestamp.timestamp() }.dump().as_bytes());
        match udp_socket.recv(&mut recv_buf) {
          Ok(_data_read) => {
            if let Ok(json_data) = json::parse(&String::from_utf8(recv_buf.to_vec()).unwrap()) {
              /* Missing a step here. json_data is not just pod data */
              let pod_data = PodData::from(json_data);

            } else {
              println!("Error Parsing Json from udp socket");
            }
          },
          Err(error) => {
            println!("There was an Error Reading from the socket");
          }
        }
      }
    })?);
    Ok(())
  }

  pub fn Disconnect() {

  }
}
