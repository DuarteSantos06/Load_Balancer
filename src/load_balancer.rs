use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::{copy};
use std::thread;
use std::net::Shutdown;
use std::io;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;


struct Backend {
    addr: SocketAddr,
    healthy: AtomicBool,
    active_conns: AtomicUsize,
}


impl Backend {
    fn new(addr: &str) -> Self {
        Backend {
            addr: addr.parse().expect("invalid backend address"),
            healthy: AtomicBool::new(true),
            active_conns: AtomicUsize::new(0),
        }
    }
}

fn main() -> std::io::Result<()>{
    let backends: Arc<Vec<Backend>> = Arc::new(vec![
        Backend::new("127.0.0.1:9001"),
        Backend::new("127.0.0.1:9002"),
        Backend::new("127.0.0.1:9003"),
    ]);
    let socket = TcpListener::bind("0.0.0.0:8080")?;
    println!("a escutar em {}", socket.local_addr()?);
    let mut index=0;
    for client in socket.incoming() {
        let client = match client {
            Ok(c) => c,
            Err(e) => { eprintln!("accept failed: {e}"); continue; }
        };
        let peer = match client.peer_addr() {
            Ok(p) => p,
            Err(e) => { eprintln!("peer_addr failed: {e}"); continue; }
        };
        index = get_index(index,backends.len());
        let backends = Arc::clone(&backends);
        thread::spawn(move || {
            if let Err(e) = handle(client,&backends[index]){
                eprintln!("client {peer}: {e}");
            }
        });
    }
    Ok(())
}

fn handle(client: TcpStream,backend:&Backend) -> io::Result<()> {
    let backend_fd = TcpStream::connect(backend.addr).map_err(|e| {
        io::Error::new(e.kind(), format!("connect to backend {} failed: {e}",backend.addr))
    })?;

    let client_addr = client.peer_addr()?;
    let backend_addr = backend_fd.peer_addr()?;

    // Client -> backend
    let mut client_read = client.try_clone()?;
    let mut backend_write = backend_fd.try_clone()?;
    let client_to_backend = thread::spawn(move || {
        let result = copy(&mut client_read, &mut backend_write);
        finish(&result, &client_read, &backend_write);
        log("[up]  ", client_addr, backend_addr, &result);
    });

    // Backend -> client
    let mut backend_read = backend_fd;
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

fn get_index(prev_index: usize,n_backends: usize) -> usize {
    if prev_index < n_backends-1 { return prev_index+1 } else {return  0 };
}
