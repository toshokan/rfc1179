use async_std::io::BufReader;
use async_std::net::*;
use async_std::prelude::*;

use rfc1179::parse;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:515").await?;
    let mut incoming = listener.incoming();

    while let Some(stream) = incoming.next().await {
	eprintln!("Got connection");
	let stream = stream?;
	let (r, w) = &mut (&stream, &stream);
	let mut reader = BufReader::new(r);
	let worker = rfc1179::Worker::new(&mut reader, w);
	let log = worker.run::<parse::Parser>().await;
	eprintln!("log = {:#?}", log);
	eprintln!("End connection");
    }

    Ok(())
}
