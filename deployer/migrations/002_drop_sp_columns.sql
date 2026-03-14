-- Drop Service Principal columns — auth via managed identity (ARM_USE_MSI)
ALTER TABLE deployments DROP COLUMN IF EXISTS arm_client_id_ref;
ALTER TABLE deployments DROP COLUMN IF EXISTS arm_client_secret_ref;
ALTER TABLE deployments DROP COLUMN IF EXISTS arm_tenant_id_ref;
