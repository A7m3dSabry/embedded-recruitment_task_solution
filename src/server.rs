use crate::message;
use crate::message::{server_message, client_message};
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


    // function to handle sending messages to the client
    pub fn send(&mut self, message: server_message::Message) -> io::Result<()> {
        // create buffer to hold the encoding
        let mut buffer = Vec::new();
        message.encode(&mut buffer);

        // Send the buffer to the server
        self.stream.write_all(&buffer)?;
        self.stream.flush()?;

        println!("Sent message: {:?}", message);
        Ok(())
    }

    pub fn handle(&mut self) -> io::Result<()> {
        let mut buffer = [0; 512];
        // Read data from the client
        let bytes_read = self.stream.read(&mut buffer)?;
        if bytes_read == 0 {
            info!("Client disconnected.");
            // return error if the client disconnected to tell the thread to break its loop and exit
            return Err(io::Error::new(ErrorKind::ConnectionReset, "Client disconnected."));
        }

        match message::ClientMessage::decode(&buffer[..bytes_read]) {
            Ok(client_message) => {
                match client_message.message {
                    // Case for EchoMessage
                    Some(client_message::Message::EchoMessage(echo_message)) => {
                        info!("Received EchoMessage: {}", echo_message.content);

                        // Create the response message directly without cloning
                        let response_msg = server_message::Message::EchoMessage(echo_message);
                        // send response
                        self.send(response_msg)?;
                    },

                    // Case for AddRequest
                    Some(client_message::Message::AddRequest(add_request)) => {
                        // Perform the summation of a and b from AddRequest
                        let result = add_request.a + add_request.b;
                        info!("Received AddRequest for : a = {}, b = {}, result = {}", add_request.a, add_request.b, result);

                        // Create AddResponse with the result
                        let add_response_result = message::AddResponse { result };
                        let response_msg = server_message::Message::AddResponse(add_response_result);
                        // send response back
                        self.send(response_msg)?;
                    }

                    // Handle any unrecognized message types (though unlikely)
                    _ => {

                        error!("Received an unknown or unsupported message");
                        // note the return Err here will cause the thread to exit
                        // I considered getting an unsupported message means issue in the client side
                        return Err(io::Error::new(ErrorKind::Unsupported, "Unknown message type"));
                    }
                }
            },
            Err(e) => {
                error!("Failed to decode message: {}", e);
                // note the return Err here will cause the thread to exit
                // I considered getting an unsupported message means issue in the client side
                return Err(io::Error::new(ErrorKind::InvalidData, e));
            }
        }
        // return ok if there are no errors happened
        Ok(())
    }
}



// ====================================
pub struct Server {
    listener: TcpListener,
    is_running: Arc<AtomicBool>,
    // thread_pool: Arc<ThreadPool>,
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
        // loop while server is still running
        while self.is_running.load(Ordering::SeqCst) {
            // Listen on the port and wait for new connection
            match self.listener.accept() {
                Ok((stream, addr)) => {
                    info!("New client connected: {}", addr);

                    // copy the server running state through threads safely
                    let is_running = Arc::clone(&self.is_running);
                    // spawn new thread to serve the new client
                    thread::spawn(move || {
                        // Handle the client request
                        let mut client = Client::new(stream);
                        // Loop While server is still running
                        while is_running.load(Ordering::SeqCst) {
                            // call the client handle function to handle messages
                            match client.handle() {
                                Ok(_) => {/* DO NOTHING */},
                                Err(e) => {
                                    // Log Error happened with the client and exit the loop
                                    error!("Error handling client: {}", e);
                                    break;
                                }
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

    /// Stops the server by setting the is_running flag to false
    pub fn stop(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
            info!("Shutdown signal sent.");
        } else {
            warn!("Server was already stopped or not running.");
        }
    }
}