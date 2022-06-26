
mod cli;
fn main() {
    println!("Hello, world!");
    let options = cli::Options::parse();
    print!("options:{:?}",options)
}
