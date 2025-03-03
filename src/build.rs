use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

fn main() -> io::Result<()> {
    let src_dir = Path::new("assets");

    let out_dir = std::env::var("OUT_DIR").unwrap();

    let target_dir = Path::new(&out_dir).ancestors().nth(3).unwrap();

    let dest_dir = target_dir.join("assets");
    if let Err(e) = fs::create_dir_all(&dest_dir) {
        if e.kind() != ErrorKind::AlreadyExists {
            return Err(e);
        }
    }

    copy_dir_all(src_dir, &dest_dir)?;

    println!("cargo:rerun-if-changed=assets");

    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    for entry in src.read_dir()? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            fs::create_dir_all(&dst_path)?;
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
