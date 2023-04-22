#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use fuzz_client::Client;
use fuzz_client::GreetingProtocol;


fn main() {
    let server_address: String = String::from("127.0.0.1:8888");

    // Create instance of Client
    let client = Client::new(server_address, GreetingProtocol);

}
