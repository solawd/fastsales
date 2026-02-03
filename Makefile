.PHONY: help leptos-setup leptos-serve leptos-build run

help:
	@echo "Targets:"
	@echo "  leptos-setup  Install cargo-leptos"
	@echo "  leptos-serve  Run cargo leptos serve"
	@echo "  leptos-build  Build Leptos assets"
	@echo "  run           Build Leptos assets then run server"

leptos-setup:
	cargo install cargo-leptos

leptos-serve:
	cargo leptos serve

leptos-build:
	cargo leptos build

run:
	cargo leptos build
	cargo run
