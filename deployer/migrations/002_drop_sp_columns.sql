-- Drop Service Principal columns — auth via managed identity (ARM_USE_MSI)
-- Each statement may fail if columns already dropped; errors are ignored by the runner.
ALTER TABLE deployments DROP COLUMN arm_client_id_ref;
ALTER TABLE deployments DROP COLUMN arm_client_secret_ref;
ALTER TABLE deployments DROP COLUMN arm_tenant_id_ref;
