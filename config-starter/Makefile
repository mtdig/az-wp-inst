# =============================================================================
#  Config-starter – Makefile
#  Build targets voor de TUI configuratie generator
#
#  Auteur: Jeroen Van Renterghem
#  E-mail: jeroen.vanrenterghem@student.hogent.be
#  Datum:  2026-03-11
# =============================================================================

SHELL     := bash
APP       := config-starter
BINDIR    := bin
GOFLAGS   := -trimpath -ldflags="-s -w"

.DEFAULT_GOAL := run

.PHONY: run build all clean help

# ---------------------------------------------------------------------------
#  Targets
# ---------------------------------------------------------------------------

help: ## Toon deze hulptekst
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

run: ## Compileer en draai op huidige architectuur
	go run .

build: ## Compileer voor huidige architectuur → bin/
	@mkdir -p $(BINDIR)
	go build $(GOFLAGS) -o $(BINDIR)/$(APP) .

all: $(BINDIR)/$(APP)-linux-amd64 $(BINDIR)/$(APP)-linux-arm64 $(BINDIR)/$(APP)-darwin-arm64 $(BINDIR)/$(APP)-windows-amd64.exe ## Cross-compileer voor alle platformen

$(BINDIR)/$(APP)-linux-amd64: ## Linux x86_64
	@mkdir -p $(BINDIR)
	CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build $(GOFLAGS) -o $@ .

$(BINDIR)/$(APP)-linux-arm64: ## Linux aarch64
	@mkdir -p $(BINDIR)
	CGO_ENABLED=0 GOOS=linux GOARCH=arm64 go build $(GOFLAGS) -o $@ .

$(BINDIR)/$(APP)-darwin-arm64: ## macOS arm64
	@mkdir -p $(BINDIR)
	CGO_ENABLED=0 GOOS=darwin GOARCH=arm64 go build $(GOFLAGS) -o $@ .

$(BINDIR)/$(APP)-windows-amd64.exe: ## Windows x86_64
	@mkdir -p $(BINDIR)
	CGO_ENABLED=0 GOOS=windows GOARCH=amd64 go build $(GOFLAGS) -o $@ .

clean: ## bin/ map verwijderen
	rm -rf $(BINDIR)
