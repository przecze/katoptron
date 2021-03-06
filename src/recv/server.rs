use katoptron::{Server, PreConnection, TxError, FailExt};

use crossbeam;
use crossbeam_channel::Sender;
use hostname;
use std::net::SocketAddr;


pub fn listen(port: u16, flashes: Sender<String>) -> Result<(), TxError> {
	let name = hostname::get_hostname().unwrap_or_else(|| String::from("katoptron server"));
	let recv_addr = SocketAddr::from(([127, 0, 0, 1], port));
	let mut server = Server::listen_on(recv_addr, name)?; //errors: UnableToBindAddress

	crossbeam::scope(|scope| {
		loop {
			match server.accept() { //errors: NetworkError <- IoError
				Ok(preconn) => {
					//todo: [someday] threadpool/async
					scope.builder().name(String::from("receiver")).spawn({
						let flashes = flashes.clone();
						move || receive(preconn, flashes)
					}).unwrap();
				}
				Err(e) => {
					eprintln!("{}", e.cause_trace());
				}
			};
		}
	});

	Ok(())
}

fn receive(preconn: PreConnection, flashes: Sender<String>) {
	let (mut conn, client_name) = match preconn.await_handshake() { //errors: HandshakeFailure <- GarbageData | ProtocolDowngrade
		Ok(ret) => ret,
		Err(e) => return eprintln!("{}", e.cause_trace()),
	};

	loop {
		use katoptron::Notification;
		match conn.recv_notification() { //errors: UnexpectedHandshake | [RecvError: NetworkError <- IoError+context | DeserializationError <- BincodeError | ProtocolDowngrade]
			Ok(message) => match message {
				Notification::Popup{ msg } => {
					println!("Popup: {}", msg);
				},
				Notification::Flash{ msg } => {
					println!("Flash: {}", msg);
					flashes.send(msg);
				},
			},
			Err(e) => {
				break eprintln!("{}: {}", client_name, e.cause_trace());
			}
		}
	}
}
