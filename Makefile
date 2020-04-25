mkdir_build: build
	mkdir -p build

.PHONY: backend frontend build

frontend_deps: frontend
	cd frontend && npm install

frontend_build: frontend
	cd frontend && npm run build

dev: export STATIC_DIR = $(PWD)/frontend/public
dev:
	cd $(PWD)/frontend && npm run dev & cd $(PWD)/backend && cargo run;

build: frontend_build mkdir_build
	cp -p -r frontend/public build/public && cp -p backend/target/release build
