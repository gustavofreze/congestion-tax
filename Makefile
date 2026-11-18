# Container-first workflow. Every Cargo command runs inside the cli image, so nothing beyond Docker and GNU Make is
# installed on the host.

SHELL := /bin/bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DEFAULT_GOAL := help

PROJECT_NAME := congestion-tax
RUST_VERSION := $(shell grep -oE '[0-9]+\.[0-9]+' rust-toolchain.toml | head -1)
RUST_IMAGE   := gustavofreze/rust:$(RUST_VERSION)-cli
COMPOSE      := docker compose --env-file .env.local
NETWORK      := $(PROJECT_NAME)_default
CARGO_HOME   := /app/.cargo-home

include .env.local

TEST_DATABASE := $(DATABASE_NAME)_test

LLVM_COV_VERSION   := 0.9.1
MUTANTS_VERSION    := 27.1.0
CARGO_SORT_VERSION := 2.1.4
COVERAGE_FILTER    := --ignore-filename-regex "src/(main|app_settings|database_settings|environment_variable)\.rs"

ENSURE_CARGO_TOOL = { cargo $(1) --version 2> /dev/null | grep -q $(3) || cargo install --locked $(2)@$(3); }

MAX_LINE_LENGTH := $(shell grep -oE '^max_width = [0-9]+' rustfmt.toml | grep -oE '[0-9]+')
DOMAIN_SOURCE   := src/application/domain
DOMAIN_CRATES   := jiff|thiserror
FOREIGN_CRATES  := $(shell awk '/^\[dependencies\]/ { inside = 1; next } /^\[/ { inside = 0 } \
	inside && NF { print $$1 }' Cargo.toml | grep -vxE '$(DOMAIN_CRATES)' | tr '-' '_' | paste -sd '|')
DOMAIN_ESCAPES  := \b(?:$(FOREIGN_CRATES))::|\bcrate::(?!application::domain\b)|(?:\bsuper::){3,}

REPORT_DIR      := reports
COVERAGE_HTML   := $(REPORT_DIR)/coverage/html/index.html
MUTATION_REPORT := $(REPORT_DIR)/mutants.out/outcomes.json

TOOLS_PATH  := $(CARGO_HOME)/bin:/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
HOST_USER   := $(shell id -u):$(shell id -g)
INTERACTIVE := $(shell [[ -t 0 ]] && echo -it)
OPEN_CMD    := $(shell command -v sensible-browser 2>/dev/null || command -v xdg-open 2>/dev/null || echo open)

APP_RUN = docker run $(INTERACTIVE) --rm \
	-u $(HOST_USER) \
	-e HOME=/tmp \
	-e CARGO_HOME=$(CARGO_HOME) \
	-e PATH=$(TOOLS_PATH) \
	-v $(CURDIR):/app -w /app \
	$(RUST_IMAGE)

APP_TEST_RUN = docker run $(INTERACTIVE) --rm \
	-u $(HOST_USER) \
	-e HOME=/tmp \
	-e CARGO_HOME=$(CARGO_HOME) \
	-e PATH=$(TOOLS_PATH) \
	-e DATABASE_HOST=$(DATABASE_HOST) \
	-e DATABASE_PORT=$(DATABASE_PORT) \
	-e DATABASE_NAME=$(TEST_DATABASE) \
	-e DATABASE_USER=$(FLYWAY_USER) \
	-e DATABASE_PASSWORD=$(FLYWAY_PASSWORD) \
	--network $(NETWORK) \
	-v $(CURDIR):/app -w /app \
	$(RUST_IMAGE)

# ─── Setup and run ───────────────────────────────────────────
.PHONY: start
start: ## @setup Start the service stack
	@$(COMPOSE) up -d --build
	@$(COMPOSE) rm -f $(PROJECT_NAME)-migrate

.PHONY: stop
stop: ## @setup Stop the service stack and drop its data volume
	@$(COMPOSE) down --volumes

.PHONY: configure
configure: ## @setup Install dependencies
	@$(APP_RUN) cargo fetch --locked

.PHONY: configure-and-update
configure-and-update: ## @setup Update dependencies
	@$(APP_RUN) cargo update

# ─── Testing ─────────────────────────────────────────────────
.PHONY: tests
tests: ensure-test-database ## @test Run all tests with coverage and mutation testing
	@$(APP_TEST_RUN) sh -c '\
		rustup component add llvm-tools > /dev/null 2>&1 && \
		$(call ENSURE_CARGO_TOOL,llvm-cov,cargo-llvm-cov,$(LLVM_COV_VERSION)) && \
		$(call ENSURE_CARGO_TOOL,mutants,cargo-mutants,$(MUTANTS_VERSION)) && \
		cargo llvm-cov --locked $(COVERAGE_FILTER) --html --output-dir $(REPORT_DIR)/coverage && \
		cargo llvm-cov report $(COVERAGE_FILTER) --summary-only --fail-under-lines 100 && \
		cargo mutants --jobs 2 --cargo-arg=--locked --output $(REPORT_DIR)'

.PHONY: test-file
test-file: ensure-test-database ## @test Run a single test file without coverage (usage: make test-file FILE=tax_rules)
	@$(APP_TEST_RUN) cargo test --locked -- $(FILE)

# ─── Code review ─────────────────────────────────────────────
.PHONY: review
review: check-line-length check-domain-dependencies ## @review Run static code analysis
	@$(APP_RUN) sh -c '\
		$(call ENSURE_CARGO_TOOL,sort,cargo-sort,$(CARGO_SORT_VERSION)) && \
		cargo sort --check && \
		cargo fmt --check && \
		cargo clippy --locked --all-targets --all-features -- -D warnings'

.PHONY: fix-review
fix-review: ## @review Fix static code analysis issues
	@$(APP_RUN) sh -c '\
		$(call ENSURE_CARGO_TOOL,sort,cargo-sort,$(CARGO_SORT_VERSION)) && \
		cargo sort && \
		cargo fmt && \
		cargo clippy --locked --all-targets --all-features --fix --allow-dirty --allow-staged'

# ─── Reports ─────────────────────────────────────────────────
.PHONY: show-reports
show-reports: ## @reports Open coverage and mutation reports in the browser
	@$(OPEN_CMD) $(COVERAGE_HTML)
	@$(OPEN_CMD) $(MUTATION_REPORT)

.PHONY: show-outdated
show-outdated: ## @reports Show outdated dependencies
	@$(APP_RUN) cargo update --dry-run --verbose

# ─── Maintenance ─────────────────────────────────────────────
.PHONY: clean
clean: ## @maintenance Remove build artifacts, reports, and the dependency cache
	@rm -rf target $(REPORT_DIR) .cargo-home

.PHONY: ensure-test-database
ensure-test-database:
	@$(COMPOSE) up -d --wait $(PROJECT_NAME)-adm
	@$(COMPOSE) exec -T $(PROJECT_NAME)-adm sh -c '\
		export PGPASSWORD="$$POSTGRES_PASSWORD" && \
		dropdb --if-exists --force -U "$$POSTGRES_USER" $(TEST_DATABASE) && \
		createdb -U "$$POSTGRES_USER" $(TEST_DATABASE)'
	@$(COMPOSE) run --rm \
		-e FLYWAY_URL=jdbc:postgresql://$(DATABASE_HOST):$(DATABASE_PORT)/$(TEST_DATABASE) \
		$(PROJECT_NAME)-migrate > /dev/null

.PHONY: check-line-length
check-line-length:
	@! LC_ALL=C.UTF-8 grep -rnE --include='*.rs' '^.{$(shell echo $$(( $(MAX_LINE_LENGTH) + 1 ))),}' src tests \
		|| { echo "The Rust lines above exceed $(MAX_LINE_LENGTH) characters." >&2; exit 1; }

.PHONY: check-domain-dependencies
check-domain-dependencies:
	@! grep -rnP '$(DOMAIN_ESCAPES)' $(DOMAIN_SOURCE) \
		|| { echo "The domain may depend only on std, itself, and the crates $(DOMAIN_CRATES)." >&2; exit 1; }

# ─── Help ────────────────────────────────────────────────────
.PHONY: help
help: ## @help Display this help message
	@echo "Usage: make [target]"
	@echo ""
	@awk 'BEGIN { \
		FS = ":.*?## @"; \
		order[1]="setup"; order[2]="test"; order[3]="review"; \
		order[4]="reports"; order[5]="maintenance"; order[6]="help"; \
		title["setup"]="Setup and run"; title["test"]="Testing"; title["review"]="Code review"; \
		title["reports"]="Reports"; title["maintenance"]="Maintenance"; title["help"]="Help"; \
	} \
	/^[a-zA-Z0-9_-]+:.*## @/ { \
		split($$2, parts, " "); \
		section = parts[1]; \
		description = substr($$2, length(section) + 2); \
		entries[section] = entries[section] sprintf("\033[36m%-24s\033[0m %s\n", $$1, description); \
	} \
	END { \
		for (index_ = 1; index_ <= 6; index_++) { \
			section = order[index_]; \
			if (entries[section] != "") { \
				printf "%s\n", title[section]; \
				printf "%s\n", entries[section]; \
			} \
		} \
	}' $(MAKEFILE_LIST)
