# =============================================================================
#  Opdracht 4 – Makefile
#  Runnning Terraform (provisioning) en Ansible (configuration_management)
# =============================================================================

SHELL           := bash
.DEFAULT_GOAL   := help

# ---------------------------------------------------------------------------
# Directories
# ---------------------------------------------------------------------------
TF_DIR   := provisioning
ANSIBLE_DIR := configuration_management

# ---------------------------------------------------------------------------
# SSH sleutel voor zowel Terraform als Ansible
# ---------------------------------------------------------------------------
SSH_KEY     ?= ~/.ssh/id_ed25519_hogent
SSH_PUB_KEY ?= $(SSH_KEY).pub

# ---------------------------------------------------------------------------
# Secrets – geef mee via command of export als omgevingsvariabelen
#   make apply MYSQL_PASS=...
#   export MYSQL_PASS=... && make all
# ---------------------------------------------------------------------------
MYSQL_PASS ?= $(TF_VAR_mysql_admin_password)

# ---------------------------------------------------------------------------
# Terraform helpers
# ---------------------------------------------------------------------------
TF       := terraform -chdir=$(TF_DIR)
TF_FLAGS := -var="admin_public_key=$$(cat $(SSH_PUB_KEY))"

ifdef MYSQL_PASS
  TF_FLAGS += -var="mysql_admin_password=$(MYSQL_PASS)"
endif

# ---------------------------------------------------------------------------
# Lees Terraform outputs in als Make variabelen
# ---------------------------------------------------------------------------
define tf_output
$(shell $(TF) output -raw $(1) 2>/dev/null)
endef

# =============================================================================
#  Targets
# =============================================================================

.PHONY: help init plan apply configure all destroy clean info

help: ## Toon deze hulptekst
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

# ---------------------------------------------------------------------------
# Provisioning (Terraform)
# ---------------------------------------------------------------------------
init: ## Terraform initialiseren
	$(TF) init

plan: ## Toon wat Terraform zou wijzigen
	$(TF) plan $(TF_FLAGS)

apply: ## Azure infrastructuur aanmaken
	$(TF) apply $(TF_FLAGS) -auto-approve

# ---------------------------------------------------------------------------
# Configuratiebeheer (Ansible)
# ---------------------------------------------------------------------------
configure: ## Ansible playbook uitvoeren met Terraform outputs
	$(eval VM_IP          := $(call tf_output,public_ip_address))
	$(eval ADMIN_USER     := $(call tf_output,admin_username))
	$(eval MYSQL_FQDN     := $(call tf_output,mysql_fqdn))
	$(eval MYSQL_ADMIN     := $(call tf_output,mysql_admin_login))
	@echo "──────────────────────────────────────────────"
	@echo "  VM IP         : $(VM_IP)"
	@echo "  Admin user    : $(ADMIN_USER)"
	@echo "  MySQL FQDN    : $(MYSQL_FQDN)"
	@echo "  MySQL admin   : $(MYSQL_ADMIN)"
	@echo "──────────────────────────────────────────────"
	cd $(ANSIBLE_DIR) && uv run ansible-playbook playbooks/site.yml \
		-i "$(VM_IP)," \
		-u "$(ADMIN_USER)" \
		--private-key $(SSH_KEY) \
		-e "ansible_host=$(VM_IP)" \
		-e "tf_mysql_fqdn=$(MYSQL_FQDN)" \
		-e "tf_mysql_admin_login=$(MYSQL_ADMIN)" \
		$(if $(MYSQL_PASS),-e "db_admin_password=$(MYSQL_PASS)")

# ---------------------------------------------------------------------------
# Gecombineerde targets
# ---------------------------------------------------------------------------
all: apply configure

# ---------------------------------------------------------------------------
# cleanup
# ---------------------------------------------------------------------------
destroy: ## Alle Azure resources verwijderen
	$(TF) destroy $(TF_FLAGS) -var="mysql_admin_password=Destroy-1!" -auto-approve

destroy-vm: ## Enkel de VM en dependencies verwijderen (netwerk, compute)
	$(TF) destroy $(TF_FLAGS) -var="mysql_admin_password=Destroy-1!" -auto-approve \
		-target=module.compute \
		-target=module.network

clean: ## Lokale Terraform state & cache verwijderen
	rm -rf $(TF_DIR)/.terraform $(TF_DIR)/.terraform.lock.hcl
	rm -f  $(TF_DIR)/terraform.tfstate $(TF_DIR)/terraform.tfstate.backup

# ---------------------------------------------------------------------------
# info
# ---------------------------------------------------------------------------
info: ## Terraform outputs tonen
	@$(TF) output
