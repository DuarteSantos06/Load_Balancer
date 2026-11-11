use std::net::{TcpListener, TcpStream};
use std::io::{copy, Read, Write};


const BACKEND: &str = "127.0.0.1:9001";


fn main() -> std::io::Result<()>{
    let socket = TcpListener::bind("0.0.0.0:8080")?;
    println!("a escutar em {}", socket.local_addr()?);
    for client in socket.incoming() {
        let mut client = client?;
        if let Err(e) = handle(&mut client){
            eprintln!("error: {}", e);
        }
    }
    Ok(())
}

fn handle(client: &mut TcpStream)-> std::io::Result<()> {
    let mut backend= TcpStream::connect(BACKEND)?;

    let mut buf = [0u8;4096];
    let n = client.read(&mut buf)?;
    backend.write_all(&buf[..n])?;

    copy(&mut backend, client)?;

    Ok(())






}
