use crate::message::{client_message, server_message, AddResponse, ClientMessage};
use log::{error, info, warn};
use prost::Message;
use std::{
    io::{self, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    pub fn handle(&mut self) -> io::Result<()> {
        let mut buffer = [0; 512];
        self.stream.set_nonblocking(true)?;
        loop {
            match self.stream.read(&mut buffer) {
                Ok(0) => {
                    info!("Client disconnected.");
                    return Ok(());
                }
                Ok(bytes_read) => {
                    let request = ClientMessage::decode(&buffer[..bytes_read]);
                    assert!(request.is_ok(), "Failed to receive request",);
                    info!("Received request from the client");

                    match request.unwrap().message {
                        Some(client_message::Message::AddRequest(add_request)) => {
                            let response = AddResponse {
                                result: add_request.a + add_request.b,
                            };
                            let payload = server_message::Message::AddResponse(response.clone());
                            let mut buffer = Vec::new();
                            payload.encode(&mut buffer);
                            // let payload = response.encode_to_vec();
                            self.stream.write_all(&buffer)?;
                            self.stream.flush()?;
                        }
                        Some(client_message::Message::EchoMessage(echo)) => {
                            info!("Received EchoMessage: {}", echo.content);
                            // Echo back the message
                            let payload = server_message::Message::EchoMessage(echo.clone());
                            let mut buffer = Vec::new();
                            payload.encode(&mut buffer);
                            // let payload = response.encode_to_vec();
                            self.stream.write_all(&buffer)?;
                            self.stream.flush()?;
                        }
                        _ => panic!("Unexpected request message"),
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No data available, sleep briefly to reduce CPU usage
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(ref e)
                    if e.kind() == ErrorKind::ConnectionReset
                        || e.kind() == ErrorKind::ConnectionAborted =>
                {
                    info!("Client disconnected.");
                    return Ok(());
                }
                Err(e) => {
                    error!("Error reading from stream: {}", e);
                    return Err(e);
                }
            }
        }
    }
}

pub struct Server {
    listener: TcpListener,
    is_running: Arc<AtomicBool>,
}

impl Server {
    /// Creates a new server instance
    pub fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let is_running = Arc::new(AtomicBool::new(false));
        Ok(Server {
            listener,
            is_running,
        })
    }

    /// Runs the server, listening for incoming connections and handling them
    pub fn run(&self) -> io::Result<()> {
        self.is_running.store(true, Ordering::SeqCst); // Set the server as running
        info!("Server is running on {}", self.listener.local_addr()?);

        // Set the listener to non-blocking mode
        self.listener.set_nonblocking(true)?;

        while self.is_running.load(Ordering::SeqCst) {
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr);

                    // Handle the client request in a new thread
                    let is_running = Arc::clone(&self.is_running);
                    thread::spawn(move || {
                        let mut client = Client::new(stream);
                        while is_running.load(Ordering::SeqCst) {
                            if let Err(e) = client.handle() {
                                error!("Error handling client: {}", e);
                                break;
                            }
                        }
                    });
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No incoming connections, sleep briefly to reduce CPU usage
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }

        info!("Server stopped.");
        Ok(())
    }

    /// Stops the server by setting the `is_running` flag to `false`
    pub fn stop(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
            info!("Shutdown signal sent.");
        } else {
            warn!("Server was already stopped or not running.");
        }
    }
}
