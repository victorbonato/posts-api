include .env

.PHONY: postgres adminer run

postgres:
	docker run --rm -d --name posts_db -p 5432:5432 -e POSTGRES_PASSWORD=${DATABASE_PASSWORD} -e POSTGRES_DB=posts_db postgres

adminer:
	docker run --rm -d --name adminer --network host adminer

migrate:
	sqlx migrate run

run:
	cargo run


