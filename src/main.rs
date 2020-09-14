mod get_arbs;

fn main() {
    // println!("Hello, world!");

    println!("{:#?}", get_arbs::get_arbs().unwrap_or_else());
}
