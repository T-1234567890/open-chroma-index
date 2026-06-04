use oci_core::{Registry, encode_from_hex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = Registry::load_frozen()?;
    let result = encode_from_hex("#E85A9A", &registry)?;

    println!("{}", result.short_id);
    println!("{}", result.full_id);

    Ok(())
}
