.PHONY: docker-build docker-up docker-down docker-logs docker-shell

docker-build:
	docker compose build

docker-up:
	docker compose up -d

docker-down:
	docker compose down

docker-logs:
	docker compose logs -f

docker-restart: docker-down docker-up

docker-shell-backend:
	docker compose exec backend sh

docker-shell-ml:
	docker compose exec ml-server sh

docker-clean:
	docker compose down -v --rmi local
