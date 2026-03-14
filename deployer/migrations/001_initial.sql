CREATE TABLE IF NOT EXISTS users (
    id          CHAR(36)     PRIMARY KEY,  -- UUID v4
    username    VARCHAR(100) NOT NULL UNIQUE,
    full_name   VARCHAR(255) NOT NULL,
    password    VARCHAR(255) NOT NULL,      -- argon2 hash
    is_admin    BOOLEAN      NOT NULL DEFAULT FALSE,
    created_at  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS deployments (
    id                  CHAR(36)     PRIMARY KEY,
    user_id             CHAR(36)     NOT NULL,
    name                VARCHAR(100) NOT NULL,
    status              ENUM('draft','provisioning','provisioned','configuring','ready','failed','destroying','destroyed')
                                     NOT NULL DEFAULT 'draft',

    -- Azure / Terraform inputs
    subscription_id     VARCHAR(100) NOT NULL DEFAULT '',
    resource_group_name VARCHAR(100) NOT NULL DEFAULT 'SELab-Wordpress',
    public_ip_dns_label VARCHAR(100) NOT NULL DEFAULT '',
    mysql_server_name   VARCHAR(100) NOT NULL DEFAULT '',
    mysql_admin_login   VARCHAR(100) NOT NULL DEFAULT 'wordpressdb',
    mysql_admin_password_ref VARCHAR(255) NOT NULL DEFAULT '',  -- Vaultwarden item ID of plaintext (tijdelijk)

    -- Terraform outputs (ingevuld na apply)
    tf_public_ip        VARCHAR(50)  NOT NULL DEFAULT '',
    tf_public_fqdn      VARCHAR(255) NOT NULL DEFAULT '',
    tf_mysql_fqdn       VARCHAR(255) NOT NULL DEFAULT '',
    tf_admin_username    VARCHAR(100) NOT NULL DEFAULT '',

    -- Ansible config
    ansible_become_password_ref VARCHAR(255) NOT NULL DEFAULT '',
    wp_admin_password_ref       VARCHAR(255) NOT NULL DEFAULT '',
    db_wp_password_ref          VARCHAR(255) NOT NULL DEFAULT '',

    -- Semaphore refs (ingevuld na environment/inventory aanmaak)
    sem_environment_id  INT          NULL,
    sem_inventory_id    INT          NULL,
    sem_tf_environment_id INT        NULL,

    -- Semaphore task tracking
    sem_last_task_id    INT          NULL,

    created_at          DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at          DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE KEY uq_user_name (user_id, name)
);
