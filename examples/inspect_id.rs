use oci_core::{OciId, Registry, inspect};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = Registry::load_frozen()?;
    let id = OciId::parse_with_registry("OCI-1-46PK-236", &registry)?;
    let result = inspect(&id, &registry)?;

    println!("{:.6}", result.canonical_oklch.l);
    println!("{:.6}", result.canonical_oklch.c);
    println!("{:.6}", result.canonical_oklch.h);

    Ok(())
}
