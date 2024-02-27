use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpStream;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mio::{Events, Poll};
use mio::{Interest, Token};

const server_addr: &str = "127.0.0.1:25565";
const clients_amount: i32 = 50;
fn epoll_select(poll: &mut Poll, events: &mut Events, clients: &mut Vec<mio::net::TcpStream>) {
    poll.poll(events, None).unwrap();
    for event in events.iter() {
        event.token();
    }
}

fn epoll_benchmark(c: &mut Criterion) {
    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(128);
    let addr = server_addr.parse().unwrap();
    let mut server = mio::net::TcpListener::bind(addr).unwrap();
    let server_token = Token(0);
    poll.registry()
        .register(&mut server, server_token, Interest::READABLE)
        .unwrap();
    println!("connecting 10 clients...");
    start_bots();
    let mut clients: Vec<mio::net::TcpStream> = Vec::with_capacity(128);
    let mut acc = 0;
    while acc != clients_amount {
        if let Ok((mut stream, addr)) = server.accept() {
            poll.registry()
                .register(&mut stream, Token(clients.len()), Interest::READABLE)
                .unwrap();
            clients.push(stream);
        } else {
            continue;
        }
        acc += 1;
    }
    poll.poll(&mut events, None).unwrap();
    events.iter().for_each(|f| {});
    println!("start epoll selection");
    c.bench_function("epoll selection", |b| {
        b.iter(|| epoll_select(&mut poll, &mut events, &mut clients))
    });
}

struct Connection {
    stream: TcpStream,
}

fn std_benchmark(c: &mut Criterion) {
    let addr: SocketAddr = server_addr.parse().unwrap();
    let mut server = std::net::TcpListener::bind(addr).unwrap();
    let mut clients: Vec<Connection> = Vec::with_capacity(128);
    println!("connecting 10 clients...");
    start_bots();
    for _ in 0..clients_amount {
        let (mut stream, client_addr) = server.accept().unwrap();
        let connection = Connection { stream };
        clients.push(connection);
    }
    println!("start std selection");
    let mut buffer = &mut [0u8; 100000];
    c.bench_function("std selection", |b| {
        b.iter(|| std_select(buffer, &mut clients));
    });
}

fn std_select(buffer: &mut [u8], clients: &mut Vec<Connection>) {
    for client in clients {
        let read = client.stream.read(buffer).unwrap();
        let read_buf = &[0..read];
    }
}

fn start_bots() {
    std::thread::spawn(|| {
        let mut clients: Vec<TcpStream> = Vec::with_capacity(128);
        let addr: SocketAddr = server_addr.parse().unwrap();
        for _ in 0..clients_amount {
            let mut client = TcpStream::connect(addr).unwrap();
            clients.push(client);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        loop {
            std::thread::sleep(std::time::Duration::from_millis(1));
            for mut client in clients.iter() {
                client.write_all(&[0x00]).unwrap();
            }
        }
    });
}


criterion_group!(benches, epoll_benchmark, std_benchmark);
criterion_main!(benches);
