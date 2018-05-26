extern crate stentorian_server;

fn main() {
    if let Err(e) = stentorian_server::serve() {
        println!("{}", e.0);
    }
}
