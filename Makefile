


preinstall:
	cargo install sqlx-cli --no-default-features --features postgres,native-tls

run:
	RUST_LOG=debug cargo run

run-docker:
	docker-compose up -d

sqlx-init:
	sqlx database create

sqlx-links:
	sqlx migrate add -r links