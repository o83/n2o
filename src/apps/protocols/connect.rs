extern crate kernel;
use kernel::session_types::*;
fn server(c: Chan<(), Eps>) {
    println!("Server on");
    c.close()
}
fn client(c: Chan<(), Eps>) {
    println!("Client on");
    c.close()
}
fn main() {
    connect(server, client);
}
