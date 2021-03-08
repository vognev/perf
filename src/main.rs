extern crate argparse;
extern crate ctrlc;
extern crate human_format;

use argparse::{ArgumentParser, Store};
use std::net::{UdpSocket, SocketAddr, IpAddr, TcpListener, TcpStream};
use std::io::ErrorKind::{WouldBlock};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Instant, Duration};
use std::thread;
use std::io::Read;
use human_format::{Scales,Formatter};

const DGRAM_MAX:usize = 65536;

fn main() {
    let runs = Arc::new(AtomicBool::new(true));
    let flag = runs.clone();

    ctrlc::set_handler(move || {
        flag.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut mode = "udp".to_string();
    let mut addr = "0.0.0.0".to_string();
    let mut port:u16 = 2021;

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("perf");

        ap.refer(&mut addr).add_option(&["-a", "--addr"], Store, "Addr");
        ap.refer(&mut port).add_option(&["-p", "--port"], Store, "Port");
        ap.refer(&mut mode).add_option(&["-m", "--mode"], Store, "Mode: tcp or udp");

        ap.parse_args_or_exit();
    }

    println!("addr: {}:{}", addr, port);

    let mut start: Option<Instant> = None;
    let mut amount:u128 = 0;

    if mode == "tcp" {
        let server = TcpListener::bind(
            SocketAddr::new(IpAddr::V4(addr.parse().unwrap()), port)
        ).unwrap();

        server.set_nonblocking(true).unwrap();

        let mut stream: Option<TcpStream> = None;

        while runs.load(Ordering::SeqCst) {
            match server.accept() {
                Ok((accepted, addr)) => {
                    println!("Accepted connection from {}", addr);
                    stream = Some(accepted);
                    break;
                },
                Err(ref e) if e.kind() != WouldBlock => {
                    panic!("IO error")
                },
                _ => {
                    thread::sleep(Duration::from_millis(1));
                },
            }
        }

        let mut buf = [0; DGRAM_MAX];
        let mut socket = stream.unwrap();

        while runs.load(Ordering::SeqCst) {
            match socket.read(&mut buf) {
                Ok(size) => {
                    amount += size as u128;

                    if start.is_none() {
                        start = Some(Instant::now());
                    }
                },
                Err(ref e) if e.kind() != WouldBlock => {
                    panic!("IO error")
                },
                _ => {
                    thread::sleep(Duration::from_millis(1));
                },
            }
        }
    }

    if mode == "udp" {
        let socket = UdpSocket::bind(
            SocketAddr::new(IpAddr::V4(addr.parse().unwrap()), port)
        ).unwrap();

        socket.set_nonblocking(true).unwrap();

        let mut buf = [0; DGRAM_MAX];

        while runs.load(Ordering::SeqCst) {
            match socket.recv(&mut buf) {
                Ok(size) => {
                    amount += size as u128;

                    if start.is_none() {
                        start = Some(Instant::now());
                    }
                }
                Err(ref e) if e.kind() != WouldBlock => {
                    panic!("IO error: {}", e)
                },
                _ => {
                    thread::sleep(Duration::from_millis(1));
                },
            }
        }
    }

    {
        println!("Total amount: {}", amount);

        match start {
            Some(n) => {
                let millis = n.elapsed().as_millis();

                // warn: casting from u128 to f64
                let seconds = millis as f64 / 1000.0;

                println!("Elapsed: {}s", seconds);

                if millis > 0 {
                    println!("{}", Formatter::new()
                        .with_decimals(3)
                        .with_scales(Scales::Binary())
                        .with_units("b/s")
                        .format(amount as f64 / seconds * 8.0))
                }
            },
            None => println!("No data received")
        }
    }
}
