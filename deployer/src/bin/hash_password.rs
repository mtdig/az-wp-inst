//! CLI-hulpmiddel om een argon2 hash te genereren voor een wachtwoord.
//!
//! Gebruik: cargo run --bin hash_password -- <wachtwoord>

use argon2::{
    Argon2,
    PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};

fn main() {
    let password = std::env::args()
        .nth(1)
        .expect("Gebruik: hash_password <wachtwoord>");

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("Hashing mislukt");

    println!("{hash}");
}
