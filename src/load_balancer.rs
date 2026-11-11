use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::{copy};
use std::thread;
use std::net::Shutdown;
use std::io;


const BACKEND: &str = "127.0.0.1:9001";


fn main() -> std::io::Result<()>{
    let socket = TcpListener::bind("0.0.0.0:8080")?;
    println!("a escutar em {}", socket.local_addr()?);
    for client in socket.incoming() {
        let client = match client {
            Ok(c) => c,
            Err(e) => { eprintln!("accept failed: {e}"); continue; }
        };
        let peer = match client.peer_addr() {
            Ok(p) => p,
            Err(e) => { eprintln!("peer_addr failed: {e}"); continue; }
        };
        thread::spawn(move || {
            if let Err(e) = handle(client){
                eprintln!("client {peer}: {e}");
            }
        });
    }
    Ok(())
}

fn handle(client: TcpStream) -> io::Result<()> {
    let backend = TcpStream::connect(BACKEND).map_err(|e| {
        io::Error::new(e.kind(), format!("connect to backend {BACKEND} failed: {e}"))
    })?;

    let client_addr = client.peer_addr()?;
    let backend_addr = backend.peer_addr()?;

    // Client -> backend
    let mut client_read = client.try_clone()?;
    let mut backend_write = backend.try_clone()?;
    let client_to_backend = thread::spawn(move || {
        let result = copy(&mut client_read, &mut backend_write);
        finish(&result, &client_read, &backend_write);
        log("[up]  ", client_addr, backend_addr, &result);
    });

    // Backend -> client
    let mut backend_read = backend;
    let mut client_write = client;
    let result = copy(&mut backend_read, &mut client_write);
    finish(&result, &backend_read, &client_write);
    log("[down]", backend_addr, client_addr, &result);

    client_to_backend.join().expect("upstream thread panicked");
    Ok(())
}

fn finish(result: &io::Result<u64>, from: &TcpStream, to: &TcpStream) {
    match result {
        Ok(_) => { let _ = to.shutdown(Shutdown::Write); }
        Err(_) => {
            let _ = from.shutdown(Shutdown::Both);
            let _ = to.shutdown(Shutdown::Both);
        }
    }
}

fn log(dir: &str, from: SocketAddr, to: SocketAddr, result: &io::Result<u64>) {
    match result {
        Ok(n) => println!("{dir} {from} → {to}: {n} bytes"),
        Err(e) if is_disconnect(e) => println!("{dir} {from} → {to}: closed early ({e})"),
        Err(e) => eprintln!("{dir} {from} → {to}: error ({e})"),
    }
}

fn is_disconnect(e: &io::Error) -> bool {
    matches!(e.kind(), io::ErrorKind::ConnectionReset | io::ErrorKind::BrokenPipe)
}
