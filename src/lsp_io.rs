/// # Lsp_io
/// Module which takes a user pipe capable of reading and writing and wraps the basic
/// json protocol for sending and recieving messages 
/// 
/// TODO: I'm not really sure if this is all that stable, for example: what happens 
/// when the streams come out of sync and you're reading message when expecting 
/// the header? 
use core::str;
use std::{
    io::{BufRead, BufReader, BufWriter, Read, Write},
    str::Utf8Error,
};

use thiserror::Error;

/// # LspIoErr
/// An error has occured when trying to receieve or send content to the
/// client of the server
#[derive(Error, Debug)]
pub enum LspIoErr {
    #[error("Failed to read from user pipe due to \"{0}\"")]
    ReaderError(String),

    #[error("Failed to parse expected as token Content-Length: was missing")]
    ReadHeaderFailed,

    #[error("Failed to get the length of content to parse due to \"{0}\"")]
    ReadContentLengthFailed(String),

    #[error("Failed to convert recieved bytes from client into utf8 had error {0}")]
    Utf8ParseError(Utf8Error),

    #[error("Failed to parse recieved string into json due to \"{0}\"")]
    JsonParseError(serde_json::Error),

    #[error("Failed to write string to client")]
    WriterError(std::io::Error),
}

/// # StringReader
/// Supplying the library with a reader allows communication with
/// the client of the LSP server. Lsp clients can connect over
/// stdio and also socket connections
///
/// From here we're expected to recieve json strings and send json
/// strings back to the client
pub struct JsonReceiver<T> {
    pub reader: BufReader<T>,
}

/// # JsonSender
/// Supplying the library with a pipe to write to allows the server to
/// send messages back to the clients. This attempts to ensure the
/// format is correct so that the client can just construct messages
pub struct JsonSender<T: std::io::Write> {
    pub writer: BufWriter<T>,
}

impl<T: std::io::Read> JsonReceiver<T> {
    /// # get_next_message
    ///
    /// ## Note
    /// Blocking function as we need to wait for the entire message
    /// to send and this can block if not enough data is sent
    /// or if no data is sent.
    pub fn get_next_message(&mut self) -> Result<serde_json::Value, LspIoErr> {
        // Given the format of the lsp protocol. We should get
        // "Content-Length: XXXX \r\n\r\n"
        let mut string_buff = String::new();
        self.reader
            .read_line(&mut string_buff)
            .map_err(|e| LspIoErr::ReaderError(e.to_string()))?;

        let mut whitespace_iter = string_buff.split_whitespace();
        if whitespace_iter.next() != Some("Content-Length:") {
            return Err(LspIoErr::ReadHeaderFailed);
        }

        // Actually get the number of chars we expect the rest of the message to be
        let expected_msg_size = whitespace_iter
            .next()
            .ok_or(LspIoErr::ReadContentLengthFailed(
                "iterator failed to find value after Content-Length: token".to_string(),
            ))?
            .parse::<usize>()
            .map_err(|e| {
                LspIoErr::ReadContentLengthFailed(format!(
                    "Couldn't parse length into a number due to {}",
                    e
                ))
            })?;

        // Just got to skip the one line after the content size
        self.reader.read_line(&mut string_buff).unwrap();

        // Next We need a buffer to place this content into
        let mut recv_buff = vec![0u8; expected_msg_size];
        self.reader.read_exact(&mut recv_buff).map_err(|e| {
            LspIoErr::ReaderError(format!(
                "User reader failed to read bytes of json message. due to: {}",
                e
            ))
        })?;

        // Now try to stringize that buffer
        // TODO: Should this be some other type of string
        let str_msg = str::from_utf8(&recv_buff).map_err(|e| LspIoErr::Utf8ParseError(e))?;

        // We can now convert that borrowed string out into a json object for more interogation
        serde_json::from_str(str_msg).map_err(|e| LspIoErr::JsonParseError(e))
    }
}

impl<T: std::io::Write> JsonSender<T> {
    /// # send_message
    /// Attaches the content header to the stringified content containing the json
    /// message getting sent to the client
    pub fn send_message(&mut self, msg_to_send: &str) -> Result<(), LspIoErr> {
        self.writer
            .write(format!("Content-Length: {}\n\n", msg_to_send.len()).as_bytes())
            .map_err(|e| LspIoErr::WriterError(e))?;

        self.writer
            .write(msg_to_send.as_bytes())
            .map_err(|e| LspIoErr::WriterError(e))?;


        println!("{}", msg_to_send);
        Ok(())
    }


    pub fn send_pop_window(&mut self, popup_text: &str) -> Result<(), LspIoErr> {


        self.send_message(&format!("{{\"jsonrpc\": \"2.0\", \"id\": 2, \"method\": \"window/showMessage\", \"params\" : {{ \"type\": 3, \"message\": \"{}\"}}, }}", popup_text))

    }
}
