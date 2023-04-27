use srv::Server;


fn main() {
    let mut server = Server::new("127.0.0.1:8888");
    server.run();
}

