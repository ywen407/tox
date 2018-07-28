extern crate tox;
extern crate futures;
extern crate tokio;
extern crate tokio_codec;

#[macro_use]
extern crate log;
extern crate env_logger;

use tox::toxcore::crypto_core::*;
use tox::toxcore::tcp::server::{Server, ServerExt};

use std::time::{Duration, Instant};

use futures::prelude::*;

use tokio::net::TcpListener;
use tokio::timer::Interval;

fn main() {
    env_logger::init();
    // Server constant PK for examples/tests
    // Use `gen_keypair` to generate random keys
    let server_pk = PublicKey([177, 185, 54, 250, 10, 168, 174,
                            148, 0, 93, 99, 13, 131, 131, 239,
                            193, 129, 141, 80, 158, 50, 133, 100,
                            182, 179, 183, 234, 116, 142, 102, 53, 38]);
    let server_sk = SecretKey([74, 163, 57, 111, 32, 145, 19, 40,
                            44, 145, 233, 210, 173, 67, 88, 217,
                            140, 147, 14, 176, 106, 255, 54, 249,
                            159, 12, 18, 39, 123, 29, 125, 230]);
    let addr = "0.0.0.0:12345".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    info!("Listening on addr={}, {:?}", addr, &server_pk);

    let server = Server::new();
    let server_c = server.clone();

    let future = listener.incoming()
        .for_each(move |stream|
            server.clone().run(stream, server_sk.clone())
        )
        .map_err(|err| {
            // All tasks must have an `Error` type of `()`. This forces error
            // handling and helps avoid silencing failures.
            //
            // In our example, we are only going to log the error to STDOUT.
            println!("Server error = {:?}", err);
        });

    let interval = Duration::from_secs(1);
    let wakeups = Interval::new(Instant::now(), interval);
    let ping_sender = wakeups
        .map_err(|e| println!("TCP ping sender timer error: {:?}", e))
        .for_each(move |_instant| {
            trace!("Tcp server ping sender wake up");
            server_c.send_pings()
                .map_err(|_| ())
        });

    let future = future
        .select(ping_sender).map(|_| ()).map_err(|(e, _)| e);

    tokio::run(future);
}
