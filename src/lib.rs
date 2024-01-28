use bytes::{Buf, BufMut, BytesMut};
use error::PalconError;
use std::str;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
pub mod error;

const PACKET_LENGTH_OFFSET: i32 = 0;
const AUTH_TYPE: i32 = 3;
const COMMAND_TYPE: i32 = 2;

/// Represents a response from the server.
/// 
/// This struct contains the size, payload, and type of the response.
#[derive(Debug)]
pub struct Response {
    size: i32,
    payload: String,
    response_type: i32,
}

impl Response {
    /// Returns the payload of the response.
    pub fn payload(&self) -> &str {
        &self.payload
    }

    /// Returns the type of the response. Palworld doesn't seem to use this
    /// so it's probably safe to ignore.
    pub fn response_type(&self) -> i32 {
        self.response_type
    }

    /// Returns the size of the response.
    /// The server may lie about this.
    pub fn size(&self) -> i32 {
        self.size
    }
}

/// Represents a connection to the server.
/// 
/// This struct contains the TCP stream and the current state of the connection.
pub struct ServerConnection {
    stream: TcpStream,
    state: ConnectionState,
}

/// Represents the state of the connection.
/// 
/// The connection can be either Connected or Authenticated.
#[derive(Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Connected,
    Authenticated,
}

impl ServerConnection {
    /// Connects to the server at the given address.
    /// 
    /// This function takes an address as a string and returns a ServerConnection or a PalconError.
    pub async fn connect(address: &str) -> Result<Self, PalconError> {
        let stream = TcpStream::connect(address).await?;
        Ok(Self {
            stream,
            state: ConnectionState::Connected,
        })
    }

    /// Authenticates with the server using the provided password.
    /// 
    /// This function takes a password as a string and returns a Result.
    pub async fn authenticate(&mut self, password: &str) -> Result<(), PalconError> {
        if self.state == ConnectionState::Authenticated {
            return Err(PalconError::AlreadyAuthenticated);
        }

        self.send_packet(AUTH_TYPE, password).await?;
        let response = self.read_response().await?;
        if response.response_type == 2 {
            return Ok(());
        }
        Err(PalconError::AuthenticationError)
    }

    /// Sends a command to the server and returns the response.
    /// 
    /// This function takes a command as a string and returns a Response or a PalconError.
    pub async fn run_command(&mut self, command: &str) -> Result<Response, PalconError> {
        self.send_and_read(COMMAND_TYPE, command).await
    }

    /// Sends a packet to the server and reads the response.
    /// 
    /// This function is used internally by the ServerConnection struct.
    async fn send_and_read(
        &mut self,
        packet_type: i32,
        payload: &str,
    ) -> Result<Response, PalconError> {
        self.send_packet(packet_type, payload).await?;
        self.read_response().await
    }

    /// Sends a packet to the server.
    /// 
    /// This function is used internally by the ServerConnection struct.
    async fn send_packet(&mut self, packet_type: i32, payload: &str) -> Result<(), PalconError> {
        let mut packet = BytesMut::with_capacity(4 + 4 + payload.len() + 2);
        packet.put_i32_le(PACKET_LENGTH_OFFSET + payload.len() as i32); // Packet length
        packet.put_i32_le(0); // Request ID (0 because palworld doesn't use it nor care about it)
        packet.put_i32_le(packet_type); // Type
        packet.put_slice(payload.as_bytes());
        packet.put_i16_le(0); // Null string terminator
        self.stream.write_all(&packet).await?;

        Ok(())
    }

    /// Reads a response from the server.
    /// 
    /// This function is used internally by the ServerConnection struct.
    async fn read_response(&mut self) -> Result<Response, PalconError> {
        let mut buffer = vec![0; 4096];
        let read_timeout = Duration::from_secs(5);

        let read_future = self.stream.read(&mut buffer);
        match timeout(read_timeout, read_future).await {
            Ok(Ok(n)) => {
                let response = Self::decode_response(&buffer[..n]);
                Ok(response)
            }
            Ok(Err(e)) => Err(PalconError::from(e)),
            Err(_) => {
                Err(PalconError::TimeoutError)
            }
        }
    }

    /// Decodes a response from the server.
    /// 
    /// This function is used internally by the ServerConnection struct.
    fn decode_response(buffer: &[u8]) -> Response {
        let mut buf = buffer;
        let response_size = buf.get_i32_le();
        // These go unused
        let _response_id = buf.get_i32_le();
        let response_type = buf.get_i32_le();
        let payload_end = buf
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(buf.len());
        let payload = str::from_utf8(&buf[..payload_end])
            .unwrap_or_default()
            .to_string();
        Response {
            size: response_size,
            payload,
            response_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;

    /// Quick test to make sure the library works.
    /// 
    /// This test connects to a server, authenticates, sends a command, and checks the response.
    #[tokio::test]
    async fn quick_test() {
        dotenv().ok();
        // Load the connection data from environment variables
        let server_address = env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS must be set");
        let server_password = env::var("SERVER_PASSWORD").expect("SERVER_PASSWORD must be set");
        let mut connection = ServerConnection::connect(&server_address).await.unwrap();
        let response = connection.authenticate(&server_password).await;
        assert!(response.is_ok());
        let response = connection
            .run_command("broadcast Hello!")
            .await
            .unwrap();
        assert_eq!(response.payload, "Broadcasted: Hello!\n");
        assert_eq!(response.response_type, 0);
        assert_eq!(response.size, 30);
    }
}