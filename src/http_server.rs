// src/http_server.rs - HTTP Server Runtime for Nova
// Serves Nova UI apps over HTTP with live reload

use std::net::TcpListener;
use std::io::{Read, Write};
use std::thread;

pub struct NovaServer {
    port: u16,
    html_content: String,
}

impl NovaServer {
    pub fn new(port: u16) -> Self {
        NovaServer {
            port,
            html_content: String::new(),
        }
    }
    
    pub fn set_content(&mut self, html: String) {
        self.html_content = html;
    }
    
    pub fn start(&self) {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).expect("Failed to bind to address");
        
        println!("\n🚀 Nova Server Running!");
        println!("📱 Open your browser to: http://localhost:{}", self.port);
        println!("Press Ctrl+C to stop\n");
        
        let html = self.html_content.clone();
        
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let html_clone = html.clone();
                    thread::spawn(move || {
                        let mut buffer = [0; 1024];
                        stream.read(&mut buffer).unwrap();
                        
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                            html_clone.len(),
                            html_clone
                        );
                        
                        stream.write_all(response.as_bytes()).unwrap();
                        stream.flush().unwrap();
                    });
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }
}