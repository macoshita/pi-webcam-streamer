TARGET_AARCH64 = aarch64-unknown-linux-gnu

.PHONY: all build frontend backend cross clean

all: build

frontend:
	cd frontend && bun install && bun run build

backend: frontend
	cargo build --release

cross: frontend
	rustup target add $(TARGET_AARCH64)
	cargo zigbuild --release --target $(TARGET_AARCH64)

build: backend

clean:
	cargo clean
	rm -rf frontend/build
