use std::{
    env,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

use sha1::{Digest, Sha1};

const SHA1_HEX_STRING_LENGTH: usize = 40;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage:");
        println!("sha_cracher: <wordlist.txt> <sha1_hash>");
        return Ok(());
    }

    let hash_to_crack = args[2].trim();
    if hash_to_crack.len() != SHA1_HEX_STRING_LENGTH {
        return Err("sha1 hash is not valid".into());
    }

    let wordlist_file = File::open(&args[1])?;
    let reader = BufReader::new(&wordlist_file);

    for line in reader.lines() {
        let line = line?;
        let common_password = line.trim();

        if hash_to_crack == hex::encode(Sha1::digest(common_password.as_bytes())) {
            print!("Password found: {common_password}");
            return Ok(());
        }
    }

    Ok(())
}
