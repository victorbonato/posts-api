include .env

.PHONY: run clean

run:
	docker compose up --build --wait && docker build --network=host -t img_rust_api . && docker run -d --network=host --name rust_api img_rust_api

clean:
	docker container stop posts_api_postgres rust_api && docker container rm posts_api_postgres rust_api && docker rmi img_rust_api



