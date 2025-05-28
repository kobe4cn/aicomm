DOCKER=podman
REGISTRY=docker.io/kobe4cn# 替换为您的镜像仓库地址

build:
	$(DOCKER) manifest create $(REGISTRY)/chat_server:latest || true
	$(DOCKER) build \
		-t $(REGISTRY)/chat_server-amd64:latest \
		--build-arg APP_NAME=chat_server \
		--build-arg APP_PORT=6688 \
		-f Dockerfile_amd .
	$(DOCKER) build \
		-t $(REGISTRY)/chat_server-arm64:latest \
		--build-arg APP_NAME=chat_server \
		--build-arg APP_PORT=6688 \
		-f Dockerfile_arm .
	$(DOCKER) manifest add $(REGISTRY)/chat_server:latest $(REGISTRY)/chat_server-amd64:latest
	$(DOCKER) manifest add $(REGISTRY)/chat_server:latest $(REGISTRY)/chat_server-arm64:latest
	$(DOCKER) manifest push $(REGISTRY)/chat_server:latest

	$(DOCKER) manifest create $(REGISTRY)/analytics_server:latest || true
	$(DOCKER) build \
		-t $(REGISTRY)/analytics_server-amd64:latest \
		--build-arg APP_NAME=analytics_server \
		--build-arg APP_PORT=6690 \
		-f Dockerfile_amd .
	$(DOCKER) build \
		-t $(REGISTRY)/analytics_server-arm64:latest \
		--build-arg APP_NAME=analytics_server \
		--build-arg APP_PORT=6690 \
		-f Dockerfile_arm .
	$(DOCKER) manifest add $(REGISTRY)/analytics_server:latest $(REGISTRY)/analytics_server-amd64:latest
	$(DOCKER) manifest add $(REGISTRY)/analytics_server:latest $(REGISTRY)/analytics_server-arm64:latest
	$(DOCKER) manifest push $(REGISTRY)/analytics_server:latest

	$(DOCKER) manifest create $(REGISTRY)/bot_server:latest || true
	$(DOCKER) build \
		-t $(REGISTRY)/bot_server-amd64:latest \
		--build-arg APP_NAME=bot_server \
		--build-arg APP_PORT=6689 \
		-f Dockerfile_amd .
	$(DOCKER) build \
		-t $(REGISTRY)/bot_server-arm64:latest \
		--build-arg APP_NAME=bot_server \
		--build-arg APP_PORT=6689 \
		-f Dockerfile_arm .
	$(DOCKER) manifest add $(REGISTRY)/bot_server:latest $(REGISTRY)/bot_server-amd64:latest
	$(DOCKER) manifest add $(REGISTRY)/bot_server:latest $(REGISTRY)/bot_server-arm64:latest
	$(DOCKER) manifest push $(REGISTRY)/bot_server:latest

	$(DOCKER) manifest create $(REGISTRY)/notify_server:latest || true
	$(DOCKER) build \
		-t $(REGISTRY)/notify_server-amd64:latest \
		--build-arg APP_NAME=notify_server \
		--build-arg APP_PORT=6687 \
		-f Dockerfile_amd .
	$(DOCKER) build \
		-t $(REGISTRY)/notify_server-arm64:latest \
		--build-arg APP_NAME=notify_server \
		--build-arg APP_PORT=6687 \
		-f Dockerfile_arm .
	$(DOCKER) manifest add $(REGISTRY)/notify_server:latest $(REGISTRY)/notify_server-amd64:latest
	$(DOCKER) manifest add $(REGISTRY)/notify_server:latest $(REGISTRY)/notify_server-arm64:latest
	$(DOCKER) manifest push $(REGISTRY)/notify_server:latest

run:
	$(DOCKER) container prune -f
	$(DOCKER) run --entrypoint /app/chat_server --network host -d --name chat_server \
		--env OPENAI_API_KEY=$(OPENAI_API_KEY) \
		-p 6688:6688 \
		--mount type=bind,source=$(PWD)/fixtures/app.yaml,target=/app/app.yaml,readonly \
		$(REGISTRY)/chat_server:latest

	$(DOCKER) run --entrypoint /app/analytics_server --network host -d --name analytics_server \
		-p 6690:6690 \
		--mount type=bind,source=$(PWD)/fixtures/analytics.yaml,target=/app/analytics.yaml,readonly \
		$(REGISTRY)/analytics_server:latest

	$(DOCKER) run --entrypoint /app/bot_server --network host -d --name bot_server \
		--env OPENAI_API_KEY=$(OPENAI_API_KEY) \
		-p 6689:6689 \
		--mount type=bind,source=$(PWD)/fixtures/bot.yaml,target=/app/bot.yaml,readonly \
		$(REGISTRY)/bot_server:latest

	$(DOCKER) run --entrypoint /app/notify_server --network host -d --name notify_server \
		-p 6687:6687 \
		--mount type=bind,source=$(PWD)/fixtures/notify.yaml,target=/app/notify.yaml,readonly \
		$(REGISTRY)/notify_server:latest
