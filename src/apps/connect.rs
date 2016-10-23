extern crate kernel;
use kernel::abstractions::session_types::*;
fn server(c: Chan<(), Eps>) {
    c.close()
}
fn client(c: Chan<(), Eps>) {
    c.close()
}
fn main() {
    connect(server, client);
}
