DOCKER=podman

build:
	$(DOCKER) build -t chat_server:latest --build-arg APP_NAME=chat_server --build-arg APP_PORT=6688 .
	$(DOCKER) build -t analytics_server:latest --build-arg APP_NAME=analytics_server --build-arg APP_PORT=6690 .
	$(DOCKER) build -t bot_server:latest --build-arg APP_NAME=bot_server --build-arg APP_PORT=6689 .
	$(DOCKER) build -t notify_server:latest --build-arg APP_NAME=notify_server --build-arg APP_PORT=6687 .

run:
	$(DOCKER) container prune -f
	$(DOCKER) run --entrypoint /app/chat_server --network host -d --name chat_server \
		--env OPENAI_API_KEY=$(OPENAI_API_KEY) \
		-p 6688:6688 \
		--mount type=bind,source=$(PWD)/fixtures/app.yaml,target=/app/app.yaml,readonly \
		localhost/chat_server:latest

	$(DOCKER) run --entrypoint /app/analytics_server --network host -d --name analytics_server \
		-p 6690:6690 \
		--mount type=bind,source=$(PWD)/fixtures/analytics.yaml,target=/app/analytics.yaml,readonly \
		localhost/analytics_server:latest

	$(DOCKER) run --entrypoint /app/bot_server --network host -d --name bot_server \
		--env OPENAI_API_KEY=$(OPENAI_API_KEY) \
		-p 6689:6689 \
		--mount type=bind,source=$(PWD)/fixtures/bot.yaml,target=/app/bot.yaml,readonly \
		localhost/bot_server:latest

	$(DOCKER) run --entrypoint /app/notify_server --network host -d --name notify_server \
		-p 6687:6687 \
		--mount type=bind,source=$(PWD)/fixtures/notify.yaml,target=/app/notify.yaml,readonly \
		localhost/notify_server:latest
