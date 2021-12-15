CONTAINER_NAME=gpi-remapper-armv7

# check if rust builder is running
RUNNING = $(shell docker ps --format '{{.Names}}' | grep -w $(CONTAINER_NAME) -m1)

# build the rust image with cross compile dependencies and runs a container in background
start-env:
ifneq ($(RUNNING), $(CONTAINER_NAME))
	docker build -t $(CONTAINER_NAME) .
	docker run --name $(CONTAINER_NAME) --volume $(PWD):/usr/src/app --detach $(CONTAINER_NAME)
endif

# stops the docker container
stop-env:
ifeq ($(RUNNING), $(CONTAINER_NAME))
	docker stop $(CONTAINER_NAME) && docker rm $(CONTAINER_NAME)
endif

# setting linker in .cargo/config doesn't seem to work
build: start-env
	docker exec -it $(CONTAINER_NAME) /bin/bash -c \
		"RUSTFLAGS='-C linker=arm-linux-gnueabihf-gcc' cargo build --target armv7-unknown-linux-gnueabihf --release"

