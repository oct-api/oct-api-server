DEBUG ?= 1
IMG_TAG ?= $(shell git describe --tags --always)
DOCKER_IMAGE = registry.gitlab.com/famzheng/oct-api:$(IMG_TAG)
OCT_ENV ?= preview
TARGET = $(if $(DEBUG),debug,release)
.PHONY: FORCE

all: ui api docs FORCE

docs: FORCE
	cd doc && mkdocs build

doc-serve: FORCE
	cd doc && mkdocs serve -a 127.0.0.1:8088

api: FORCE
	cargo build $(if $(DEBUG),,--release)

release: FORCE
	cargo build --release

ui: FORCE
	cd ui && npm install && npm run build

run: FORCE
	cargo run -- -U

test: FORCE
	cargo test -- $(if $V,--nocapture)
	set -e; for x in tests/test*.py; do $$x; done

build: all FORCE
	mkdir -p build
	cp target/$(TARGET)/oct-api build
	cp -r ui/dist doc orm build
	cp Dockerfile build

docker:
	docker build -t $(DOCKER_IMAGE) build

deploy: FORCE
	docker push $(DOCKER_IMAGE)
	./scripts/deploy -i $(IMG_TAG) -E $(OCT_ENV) $(if $(TOKEN),-t $(TOKEN))
