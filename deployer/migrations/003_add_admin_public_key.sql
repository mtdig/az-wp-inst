-- Voeg SSH publieke sleutel kolom toe aan deployments
ALTER TABLE deployments ADD COLUMN admin_public_key TEXT NOT NULL DEFAULT '';
