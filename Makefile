include .env

.PHONY: postgres adminer run

run:
	docker compose up --build --wait && sqlx migrate run && cargo run


