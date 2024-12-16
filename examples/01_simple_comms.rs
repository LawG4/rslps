/// # 01_simple_comms
/// 
/// Shows the simple method used to communicate between the servers
/// it shows how to pass an IO pipe and accept a connection, send a 
/// message and then wait for a disconnection 
use std::{
    io::{BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
    time::Duration,
};

use rslps::{lsp_io::JsonReceiver, lsp_io::JsonSender};

fn main() -> std::io::Result<()> {
    let listener = std::net::TcpListener::bind("127.0.0.1:6502")?;
    let stream = listener_block_for_connection(listener).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();

    let mut json_recver = JsonReceiver {
        reader: BufReader::new(stream.try_clone().unwrap()),
    };
    let mut json_sender = JsonSender {
        writer: BufWriter::new(stream),
    };

    // Get the initialization message from the server
    println!("\"{}\"", json_recver.get_next_message().unwrap());

    // Now send the response
    let json_string = serde_json::json!(
    {
        "id": 1,
        "jsonrpc": "2.0",
        "result": {
            "capabilities" : {},
            "serverInfo" :{
                "name": "Rust Implementation"
            }
        }
    });
    json_sender.send_message(&json_string.to_string()).unwrap();
    json_sender.writer.flush().unwrap();

    // The server will then send a initialized response
    println!("{}", json_recver.get_next_message().unwrap());

    // Try and send a popup message
    let json_string = serde_json::json!(
    {
        "jsonrpc": "2.0",
        "method": "window/showMessage",
        "params": {
            "type": 1,
            "message": "Hewwo from rust"
        }
    });
    json_sender.send_message(&json_string.to_string()).unwrap();
    json_sender.writer.flush().unwrap();

    // Loop waiting for the user to quit the server
    loop {
        match json_recver.get_next_message() {
            Ok(content) => {
                match content["method"].as_str() {
                    Some("shutdown") => {
                        println!("Client has asked for shutdown!");
                        break;
                    }
                    _ => {}
                }

                println!("{}", content)
            }
            Err(_) => {}
        }
    }

    // Yay exit
    Ok(())
}

fn listener_block_for_connection(listener: TcpListener) -> Option<TcpStream> {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                return Some(stream);
            }
            Err(err) => {
                println!("Socket error {:?}", err);
            }
        }
    }
    None
}
