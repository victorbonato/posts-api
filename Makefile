include .env

.PHONY: run

run:
	docker compose up --build --wait && sqlx migrate run && cargo run


