// Pack the assets
fn main() {    
    let path = cflake_engine_packer::pack("assets");
    println!("Le build");
    println!("{:?}", path);
}