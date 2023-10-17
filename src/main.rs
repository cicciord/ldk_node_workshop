use ldk_node::bitcoin::secp256k1::PublicKey;
use ldk_node::bitcoin::Network;
use ldk_node::lightning_invoice::Invoice;
use ldk_node::{Builder, Config, NetAddress};
use std::str::FromStr;

// sample constants
const ESPLORA_SERVER_URL: &str = "https://mempool.space/testnet/api";
const NODE_ID: &str = "0296e20fa99d2940b8b00117e65d27003f0d8f81a0c960f71a5466d1aadf5ea5ea";
const NODE_ADDR: &str = "69.59.18.82:9735";
const CHANNEL_AMOUNT_SATS: u64 = 10000; 
const INVOICE_STR: &str = "lnbc100p1psj9jhxdqud3jxktt5w46x7unfv9kz6mn0v3jsnp4q0d3p2sfluzdx45tqcs\
h2pu5qc7lgq0xs578ngs6s0s68ua4h7cvspp5q6rmq35js88zp5dvwrv9m459tnk2zunwj5jalqtyxqulh0l\
5gflssp5nf55ny5gcrfl30xuhzj3nphgj27rstekmr9fw3ny5989s300gyus9qyysgqcqpcrzjqw2sxwe993\
h5pcm4dxzpvttgza8zhkqxpgffcrf5v25nwpr3cmfg7z54kuqq8rgqqqqqqqq2qqqqq9qq9qrzjqd0ylaqcl\
j9424x9m8h2vcukcgnm6s56xfgu3j78zyqzhgs4hlpzvznlugqq9vsqqqqqqqlgqqqqqeqq9qrzjqwldmj9d\
ha74df76zhx6l9we0vjdquygcdt3kssupehe64g6yyp5yz5rhuqqwccqqyqqqqlgqqqqjcqq9qrzjqf9e58a\
guqr0rcun0ajlvmzq3ek63cw2w282gv3z5uupmuwvgjtq2z55qsqqg6qqqyqqqrtnqqqzq3cqygrzjqvphms\
ywntrrhqjcraumvc4y6r8v4z5v593trte429v4hredj7ms5z52usqq9ngqqqqqqqlgqqqqqqgq9qrzjq2v0v\
p62g49p7569ev48cmulecsxe59lvaw3wlxm7r982zxa9zzj7z5l0cqqxusqqyqqqqlgqqqqqzsqygarl9fh3\
8s0gyuxjjgux34w75dnc6xp2l35j7es3jd4ugt3lu0xzre26yg5m7ke54n2d5sym4xcmxtl8238xxvw5h5h5\
j5r6drg6k6zcqj0fcwg";

fn main() {
	// Welcome! Please run through the the following steps.
	// "..." marks where you'll need to add code.

	// Setup Config
	let mut config = Config::default();
	config.network = Network::Testnet;

	// Configure Esplora URL) - Setup Builder from config and build() node
	let mut builder = Builder::from_config(config);
	builder.set_esplora_server(ESPLORA_SERVER_URL.to_string());
	let node = builder.build().unwrap();

	// Start LDK Node
	node.start().unwrap();

	// Get a new funding address and have it funded via the faucet
	let funding_address = node.new_onchain_address().unwrap();
	println!("New address created: {}", funding_address);
	println!("Send funds to this address before trying to open a channel. ({}sat needed)", CHANNEL_AMOUNT_SATS);
	pause();

	// Open channel to our node (see details above)
	let node_id = PublicKey::from_str(NODE_ID).unwrap();
	let node_addr = NetAddress::from_str(NODE_ADDR).unwrap();
	
	let _ = node.connect_open_channel(node_id, node_addr, CHANNEL_AMOUNT_SATS, None, None, false);

	//==============================================
	// We're now waiting for the channel to be confirmed:
	match node.wait_next_event() {
		ldk_node::Event::ChannelPending { channel_id, counterparty_node_id, .. } => println!(
			"New channel with {} pending confirmation: {:?}",
			counterparty_node_id, channel_id
		),
		e => println!("Unexpected event: {:?}", e),
	}
	node.event_handled();

	// Wait for 6 blocks (a 15 secs)
	std::thread::sleep(std::time::Duration::from_secs(90));
	node.sync_wallets().unwrap();

	match node.wait_next_event() {
		ldk_node::Event::ChannelReady { channel_id, .. } => {
			println!("Channel {:?} is ready to be used!", channel_id)
		}
		e => println!("Unexpected event: {:?}", e),
	}
	node.event_handled();
	//==============================================

	// // Parse invoice (Invoice::from_str)
	// let invoice = Invoice::from_str(INVOICE_STR).unwrap();
	
	// // Pay invoice
	// node.send_payment(&invoice).unwrap();
	let _ = node.send_spontaneous_payment(50, PublicKey::from_str(NODE_ID).unwrap());

	//==============================================
	// Wait for the payment to be successful.
	match node.wait_next_event() {
		ldk_node::Event::PaymentSuccessful { payment_hash } => {
			println!("Payment with hash {:?} successful!", payment_hash)
		}
		e => println!("Unexpected event: {:?}", e),
	}
	node.event_handled();
	node.stop().unwrap();
}

fn pause() {
	use std::io;
	use std::io::prelude::*;

	let mut stdin = io::stdin();
	let mut stdout = io::stdout();

	// We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
	write!(stdout, "Press any key to continue...").unwrap();
	stdout.flush().unwrap();

	// Read a single byte and discard
	let _ = stdin.read(&mut [0u8]).unwrap();
}
