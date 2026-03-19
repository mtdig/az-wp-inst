# =============================================================================
#  Opdracht 4 – Makefile
#  Runnning Terraform (provisioning) en Ansible (configuration_management)
#
#  Auteur: Jeroen Van Renterghem
#  E-mail: jeroen.vanrenterghem@student.hogent.be
#  Datum:  2026-03-11
#  Repo:   https://github.com/mtdig/az-wp-inst
# =============================================================================

SHELL           := bash
.DEFAULT_GOAL   := help

# ---------------------------------------------------------------------------
# Directories
# ---------------------------------------------------------------------------
TF_DIR   := provisioning
ANSIBLE_DIR := configuration_management
TF_VARS_FILE := ../terraform.tfvars.json
ANSIBLE_VARS_FILE := ../ansible_vars.json

# ---------------------------------------------------------------------------
# SSH sleutel voor zowel Terraform als Ansible
# ---------------------------------------------------------------------------
SSH_KEY     ?= ~/.ssh/id_ed25519_hogent
SSH_PUB_KEY ?= $(SSH_KEY).pub

# ---------------------------------------------------------------------------
# Terraform helpers
# ---------------------------------------------------------------------------
TF       := terraform -chdir=$(TF_DIR)
TF_FLAGS := -var-file="$(TF_VARS_FILE)" -var="admin_public_key=$$(cat $(SSH_PUB_KEY))"

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
.PHONY: aws-init aws-plan aws-apply aws-configure aws-all aws-destroy aws-clean aws-info

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
configure: ## ansible playbook uitvoeren met Terraform outputs
	$(eval VM_IP          := $(call tf_output,public_ip_address))
	$(eval ADMIN_USER     := $(call tf_output,admin_username))
	$(eval MYSQL_FQDN     := $(call tf_output,mysql_fqdn))
	$(eval MYSQL_ADMIN     := $(call tf_output,mysql_admin_login))
	$(eval PUBLIC_FQDN    := $(call tf_output,public_fqdn))
	@echo "──────────────────────────────────────────────"
	@echo "  VM IP         : $(VM_IP)"
	@echo "  Admin user    : $(ADMIN_USER)"
	@echo "  Public FQDN   : $(PUBLIC_FQDN)"
	@echo "  MySQL FQDN    : $(MYSQL_FQDN)"
	@echo "  MySQL admin   : $(MYSQL_ADMIN)"
	@echo "──────────────────────────────────────────────"
	cd $(ANSIBLE_DIR) && uv run ansible-playbook playbooks/site.yml \
		-i "$(VM_IP)," \
		-u "$(ADMIN_USER)" \
		--private-key $(SSH_KEY) \
		-e @$(ANSIBLE_VARS_FILE) \
		-e "ansible_host=$(VM_IP)" \
		-e "tf_public_fqdn=$(PUBLIC_FQDN)" \
		-e "tf_mysql_fqdn=$(MYSQL_FQDN)" \
		-e "tf_mysql_admin_login=$(MYSQL_ADMIN)" \
		-e "db_admin_password=$$(jq -r .mysql_admin_password $(TF_VARS_FILE))"

# ---------------------------------------------------------------------------
# Gecombineerde targets
# ---------------------------------------------------------------------------
all: apply configure

# ---------------------------------------------------------------------------
# cleanup
# ---------------------------------------------------------------------------
destroy: ## Alle Azure resources verwijderen
	$(TF) destroy $(TF_FLAGS) -auto-approve

destroy-vm: ## Enkel de VM en dependencies verwijderen (netwerk, compute)
	$(TF) destroy $(TF_FLAGS) -auto-approve \
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

# =============================================================================
#  AWS – OS-NPE op EC2
# =============================================================================
AWS_TF_DIR   := provisioning/aws
AWS_TF       := terraform -chdir=$(AWS_TF_DIR)
AWS_TF_VARS  := ../../aws_terraform.tfvars.json
AWS_TF_FLAGS := -var-file="$(AWS_TF_VARS)" -var="admin_public_key=$$(cat $(SSH_PUB_KEY))"
AWS_ANSIBLE_VARS := ../ansible_vars.json

define aws_tf_output
$(shell $(AWS_TF) output -raw $(1) 2>/dev/null)
endef

aws-init: ## AWS Terraform initialiseren
	$(AWS_TF) init

aws-plan: ## Toon wat AWS Terraform zou wijzigen
	$(AWS_TF) plan $(AWS_TF_FLAGS)

aws-apply: ## AWS EC2 instance aanmaken
	$(AWS_TF) apply $(AWS_TF_FLAGS) -auto-approve

aws-configure: ## AWS VM configureren met Ansible (OS-NPE stack)
	$(eval AWS_VM_IP     := $(call aws_tf_output,public_ip_address))
	$(eval AWS_ADMIN     := $(call aws_tf_output,admin_username))
	$(eval AWS_DNS       := $(call aws_tf_output,public_dns))
	@echo "──────────────────────────────────────────────"
	@echo "  AWS VM IP     : $(AWS_VM_IP)"
	@echo "  Admin user    : $(AWS_ADMIN)"
	@echo "  Public DNS    : $(AWS_DNS)"
	@echo "──────────────────────────────────────────────"
	cd $(ANSIBLE_DIR) && uv run ansible-playbook playbooks/aws-os-npe.yml \
		-i "$(AWS_VM_IP)," \
		-u "$(AWS_ADMIN)" \
		--private-key $(SSH_KEY) \
		-e @$(AWS_ANSIBLE_VARS) \
		-e '{"ansible_host":"$(AWS_VM_IP)","aws_public_dns":"$(AWS_DNS)","admin_public_key":"'"$$(cat $(SSH_PUB_KEY))"'","ssh_key":"$(SSH_KEY)"}'

aws-all: aws-apply aws-configure ## AWS apply + configure

aws-destroy: ## AWS resources verwijderen
	$(AWS_TF) destroy $(AWS_TF_FLAGS) -auto-approve

aws-clean: ## AWS Terraform state verwijderen
	rm -rf $(AWS_TF_DIR)/.terraform $(AWS_TF_DIR)/.terraform.lock.hcl
	rm -f  $(AWS_TF_DIR)/terraform.tfstate $(AWS_TF_DIR)/terraform.tfstate.backup

aws-info: ## AWS Terraform outputs tonen
	@$(AWS_TF) output
