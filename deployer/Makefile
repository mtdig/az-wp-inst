.PHONY: build run dev hash-password

# Lokaal bouwen (voor development)
build:
	cargo build

# Release build voor de huidige architectuur
release:
	cargo build --release

# Lokaal draaien
run:
	cargo run

# Development mode met auto-reload (vereist cargo-watch)
dev:
	cargo watch -x run

# Wachtwoord hashen voor gebruikers in de database
# Gebruik: make hash-password PW=geheim
hash-password:
	cargo run --bin hash_password -- "$(PW)"
