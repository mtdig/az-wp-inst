# WordPress Ansible Installatie

Een vereenvoudigde automatische installatie van een WordPress stack op een Azure VM met Apache, PHP en een **Azure MySQL Flexible Server** als database backend.

## Introductie

[Ansible](https://docs.ansible.com/) is een open-source automatiseringstool voor configuratiebeheer, applicatie-deployment en task automation. In tegenstelling tot andere tools zoals Puppet of Chef, werkt Ansible **agentless**: er hoeft geen software geïnstalleerd te worden op de doelservers.

Ansible maakt verbinding via **SSH** en voert taken uit door Python-modules naar de remote host te kopiëren en daar uit te voeren. Dit maakt het eenvoudig om te beginnen — als je via SSH kunt inloggen, kan Ansible het beheren.

De configuratie wordt geschreven in **YAML** bestanden (playbooks) die beschrijven welke taken uitgevoerd moeten worden. Ansible is **idempotent**: je kunt een playbook meerdere keren uitvoeren en alleen wijzigingen worden doorgevoerd.

Meer informatie: [Ansible Documentation](https://docs.ansible.com/ansible/latest/index.html)

## Installatie overzicht

### Role: common (basis beveiliging)

1. **SSH hardening**
   
Wachtwoord-authenticatie wordt uitgeschakeld.  Enkel key-based authenticatie wordt toegelaten.

   - `PasswordAuthentication no`
   - `PubkeyAuthentication yes`
   - `PermitRootLogin prohibit-password`
   - `ChallengeResponseAuthentication no`

2. **Packages installeren**
   - `ufw` (firewall)
   - `fail2ban` (brute-force bescherming)
   - `neofetch` (systeeminfo)
   - `wp-cli` (command line management tool voor WP, voor bvb updates te schedulen via cron jobs)

3. **UFW firewall configuratie**
   - Standaard: inkomend verkeer deny, uitgaand allow
   - Poort 22 (SSH) toegelaten
   - Poort 80 (HTTP) toegelaten
   - Poort 443 (HTTPS) toegelaten

4. **fail2ban configuratie**
   - 3 pogingen voor ban
   - 1 uur ban tijd
   - Localhost en ansible host uitgezonderd

5. **neofetch toevoegen aan .bashrc**
   - Voor root en ansible user

### Role: mysql_client (Azure MySQL connectie)

1. **MySQL client en PyMySQL installeren**
   - `mysql-client`
   - `python3-pymysql` (nodig voor Ansible)

2. **Wachten tot Azure MySQL bereikbaar is**

3. **WordPress database aanmaken op Azure MySQL**
   - Database: `wordpress`
   - Verbinding via SSL naar `jr-wordpressdb.mysql.database.azure.com`

4. **WordPress database user aanmaken**
   - User: `wpuser`
   - Volledige rechten op wordpress database

### Role: wordpress (webserver)

1. **Apache en PHP installeren**
   
   - `apache2`
   - `php`, `php-mysql`, `php-curl`, `php-gd`, `php-xml`, `php-mbstring`
   - `libapache2-mod-php`

2. **Apache modules enablen**
   - `rewrite`
   - `ssl`

3. **Self-signed SSL certificaat genereren**
   - Geldig voor 365 dagen
   - Opgeslagen in `/etc/ssl/certs/` en `/etc/ssl/private/`

4. **WordPress downloaden en uitpakken**
   - Van wordpress.org
   - Naar `/var/www/wordpress`

5. **wp-config.php aanmaken**
   - Database credentials
   - Security keys

6. **Apache virtual host configureren**
   - HTTP redirect naar HTTPS
   - SSL met self-signed certificaat

7. **Default site uitschakelen**

8. **WP-CLI installeren**
   - Command-line interface voor WordPress
   - Geïnstalleerd in `/usr/local/bin/wp`
   - Alias `wp` met `--path=/var/www/wordpress` in `.bashrc`

9. **Ansible user toevoegen aan www-data groep**

## Getting started

### NixOS

```bash
nix develop
```

Dit installeert automatisch Python en alle dependencies.

### Andere systemen

Vereisten:
- Python 3.10+
- uv (Blazingly fast Python package manager)

```bash
# Installeer uv (indien nog niet aanwezig)
curl -LsSf https://astral.sh/uv/install.sh | sh

# Installeer dependencies
uv sync
```

## Configuratie

### Inventory

Pas `inventory.yml` aan met je eigen hosts:

```yaml
all:
  hosts:
    wpdeb:
      ansible_host: 192.168.122.40
      ansible_user: osboxes
      ansible_python_interpreter: /usr/bin/python3
```

### Gecentraliseerde variabelen

Alle user-specifieke variabelen (ook secrets) staan in één rootbestand:

- `../user_vars.tfvars.json` (lokaal, in `.gitignore`)
- `../user_vars.tfvars.json.example` (template)

Maak het lokale bestand aan met:

```bash
cp ../user_vars.tfvars.json.example ../user_vars.tfvars.json
```

Pas daarna je waarden aan (subscription, domein, wachtwoorden, e-mail, ...).

## Uitvoeren

```bash
# Volledige installatie
uv run ansible-playbook playbooks/site.yml

# Alleen specifieke host
uv run ansible-playbook playbooks/site.yml --limit jouw_server

# Zonder common role (skip SSH hardening, UFW, fail2ban)
uv run ansible-playbook playbooks/site.yml -e skip_common=true

# Dry-run (check mode)
uv run ansible-playbook playbooks/site.yml --check

# Met verbose output
uv run ansible-playbook playbooks/site.yml -v
```

## Structuur

```
.
├── ansible.cfg          # Ansible configuratie
├── inventory.yml        # Host definities
├── ../user_vars.tfvars.json  # Gecentraliseerde user variabelen (root, niet in git)
├── playbooks/
│   └── site.yml         # Hoofdplaybook
└── roles/
    ├── common/          # SSH, UFW, fail2ban, neofetch
    ├── mysql_client/    # Azure MySQL client, DB & user provisioning
    └── wordpress/       # Apache, PHP, WordPress
```

## Na installatie

Na succesvolle installatie verschijnt een bericht met de URL om WordPress te configureren via de webbrowser.

## demo

Volledige run, na fresh install debian trixie.

```bash
$ uv run ansible-playbook playbooks/site.yml --limit wpdeb

PLAY [Full WordPress stack] ****************************************************************************************************************************************************************************************

TASK [Gathering Facts] *********************************************************************************************************************************************************************************************
Sunday 08 March 2026  07:55:06 +0100 (0:00:00.008)       0:00:00.008 **********
ok: [wpdeb]

TASK [common : Harden SSH] *****************************************************************************************************************************************************************************************
Sunday 08 March 2026  07:55:07 +0100 (0:00:01.191)       0:00:01.199 **********
[WARNING]: Module remote_tmp /root/.ansible/tmp did not exist and was created with a mode of 0700, this may cause issues when running as another user. To avoid this, create the remote_tmp dir with the correct permissions manually
changed: [wpdeb] => (item={'regexp': '^#?PasswordAuthentication', 'line': 'PasswordAuthentication no'})
changed: [wpdeb] => (item={'regexp': '^#?PubkeyAuthentication', 'line': 'PubkeyAuthentication yes'})
changed: [wpdeb] => (item={'regexp': '^#?PermitRootLogin', 'line': 'PermitRootLogin prohibit-password'})
changed: [wpdeb] => (item={'regexp': '^#?ChallengeResponseAuthentication', 'line': 'ChallengeResponseAuthentication no'})

TASK [common : Install packages] ***********************************************************************************************************************************************************************************
Sunday 08 March 2026  07:55:09 +0100 (0:00:01.250)       0:00:02.449 **********
changed: [wpdeb]

TASK [common : Reset connection to pick up new binaries] ***********************************************************************************************************************************************************
Sunday 08 March 2026  07:55:30 +0100 (0:00:21.417)       0:00:23.866 **********
[WARNING]: reset_connection task does not support when conditional

TASK [common : Configure UFW] **************************************************************************************************************************************************************************************
Sunday 08 March 2026  07:55:30 +0100 (0:00:00.040)       0:00:23.907 **********
changed: [wpdeb]

TASK [common : Configure fail2ban] *********************************************************************************************************************************************************************************
Sunday 08 March 2026  07:55:33 +0100 (0:00:02.645)       0:00:26.553 **********
changed: [wpdeb]

TASK [common : Enable fail2ban] ************************************************************************************************************************************************************************************
Sunday 08 March 2026  07:55:34 +0100 (0:00:00.905)       0:00:27.459 **********
ok: [wpdeb]

TASK [common : Add neofetch to bashrc] ****************************************************************************************************************************************************************************
Sunday 08 March 2026  07:55:34 +0100 (0:00:00.617)       0:00:28.076 **********
changed: [wpdeb]

TASK [common : Add neofetch to ansible user bashrc] ***************************************************************************************************************************************************************
Sunday 08 March 2026  07:55:35 +0100 (0:00:00.302)       0:00:28.378 **********
ok: [wpdeb]

TASK [common : Allow sudo group to use sudo without password] ******************************************************************************************************************************************************
Sunday 08 March 2026  07:55:35 +0100 (0:00:00.280)       0:00:28.659 **********
changed: [wpdeb]

TASK [mariadb : Install MariaDB] ***********************************************************************************************************************************************************************************
Sunday 08 March 2026  07:55:35 +0100 (0:00:00.323)       0:00:28.982 **********
changed: [wpdeb]

TASK [mariadb : Start MariaDB] *************************************************************************************************************************************************************************************
Sunday 08 March 2026  07:56:10 +0100 (0:00:34.841)       0:01:03.824 **********
ok: [wpdeb]

TASK [mariadb : Create WordPress database] *************************************************************************************************************************************************************************
Sunday 08 March 2026  07:56:11 +0100 (0:00:00.462)       0:01:04.286 **********
[WARNING]: Deprecation warnings can be disabled by setting `deprecation_warnings=False` in ansible.cfg.
[DEPRECATION WARNING]: Importing 'to_native' from 'ansible.module_utils._text' is deprecated. This feature will be removed from ansible-core version 2.24. Use ansible.module_utils.common.text.converters instead.
changed: [wpdeb]

TASK [mariadb : Create WordPress DB user] **************************************************************************************************************************************************************************
Sunday 08 March 2026  07:56:11 +0100 (0:00:00.340)       0:01:04.627 **********
changed: [wpdeb]

TASK [wordpress : Install Apache and PHP] **************************************************************************************************************************************************************************
Sunday 08 March 2026  07:56:11 +0100 (0:00:00.390)       0:01:05.018 **********
changed: [wpdeb]

TASK [wordpress : Enable Apache modules] ***************************************************************************************************************************************************************************
Sunday 08 March 2026  07:57:05 +0100 (0:00:53.370)       0:01:58.388 **********
changed: [wpdeb] => (item=rewrite)
changed: [wpdeb] => (item=ssl)

TASK [wordpress : Generate self-signed SSL certificate] ************************************************************************************************************************************************************
Sunday 08 March 2026  07:57:06 +0100 (0:00:00.844)       0:01:59.233 **********
changed: [wpdeb]

TASK [wordpress : Download WordPress] ******************************************************************************************************************************************************************************
Sunday 08 March 2026  07:57:06 +0100 (0:00:00.317)       0:01:59.550 **********
changed: [wpdeb]

TASK [wordpress : Extract WordPress] *******************************************************************************************************************************************************************************
Sunday 08 March 2026  07:57:09 +0100 (0:00:03.048)       0:02:02.599 **********
changed: [wpdeb]

TASK [wordpress : Create wp-config.php] ****************************************************************************************************************************************************************************
Sunday 08 March 2026  07:57:11 +0100 (0:00:02.011)       0:02:04.611 **********
changed: [wpdeb]

TASK [wordpress : Create Apache vhost] *****************************************************************************************************************************************************************************
Sunday 08 March 2026  07:57:11 +0100 (0:00:00.468)       0:02:05.079 **********
changed: [wpdeb]

TASK [wordpress : Enable WordPress site] ***************************************************************************************************************************************************************************
Sunday 08 March 2026  07:57:12 +0100 (0:00:00.471)       0:02:05.551 **********
changed: [wpdeb]

TASK [wordpress : Disable default site] ****************************************************************************************************************************************************************************
Sunday 08 March 2026  07:57:12 +0100 (0:00:00.290)       0:02:05.841 **********
changed: [wpdeb]

TASK [wordpress : Download WP-CLI] *********************************************************************************************************************************************************************************
Sunday 08 March 2026  07:57:12 +0100 (0:00:00.202)       0:02:06.044 **********
changed: [wpdeb]

TASK [wordpress : Add wp alias to ansible user bashrc] *************************************************************************************************************************************************************
Sunday 08 March 2026  07:57:13 +0100 (0:00:00.849)       0:02:06.893 **********
changed: [wpdeb]

TASK [wordpress : Add ansible user to www-data group] **************************************************************************************************************************************************************
Sunday 08 March 2026  07:57:13 +0100 (0:00:00.220)       0:02:07.113 **********
changed: [wpdeb]

RUNNING HANDLER [common : Restart SSH] *****************************************************************************************************************************************************************************
Sunday 08 March 2026  07:57:14 +0100 (0:00:00.566)       0:02:07.680 **********
changed: [wpdeb]

RUNNING HANDLER [common : Restart fail2ban] ************************************************************************************************************************************************************************
Sunday 08 March 2026  07:57:14 +0100 (0:00:00.515)       0:02:08.196 **********
changed: [wpdeb]

RUNNING HANDLER [wordpress : Restart Apache] ***********************************************************************************************************************************************************************
Sunday 08 March 2026  07:57:15 +0100 (0:00:00.829)       0:02:09.026 **********
changed: [wpdeb]

TASK [Show connection info] ****************************************************************************************************************************************************************************************
Sunday 08 March 2026  07:57:16 +0100 (0:00:00.515)       0:02:09.542 **********
ok: [wpdeb] =>
    msg:
    - ==========================================
    - 'Add this line to your /etc/hosts file:'
    - 192.168.122.40    sel-opdracht4.groep99.be
    - ''
    - 'Then open: https://sel-opdracht4.groep99.be'
    - (Self-signed certificate - accept the warning)
    - ==========================================

PLAY RECAP *********************************************************************************************************************************************************************************************************
wpdeb                      : ok=29   changed=24   unreachable=0    failed=0    skipped=0    rescued=0    ignored=0


TASKS RECAP ********************************************************************************************************************************************************************************************************
Sunday 08 March 2026  07:57:16 +0100 (0:00:00.013)       0:02:09.555 **********
===============================================================================
wordpress : Install Apache and PHP ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 53.37s
mariadb : Install MariaDB ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 34.84s
common : Install packages ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 21.42s
wordpress : Download WordPress ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ 3.05s
common : Configure UFW -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 2.65s
wordpress : Extract WordPress ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 2.01s
common : Harden SSH ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 1.25s
Gathering Facts --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 1.19s
common : Configure fail2ban --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 0.91s
wordpress : Download WP-CLI --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 0.85s
wordpress : Enable Apache modules --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 0.84s
common : Restart fail2ban ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 0.83s
common : Enable fail2ban ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ 0.62s
wordpress : Add ansible user to www-data group -------------------------------------------------------------------------------------------------------------------------------------------------------------- 0.57s
common : Restart SSH ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 0.52s
wordpress : Restart Apache ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 0.52s
wordpress : Create Apache vhost ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 0.47s
wordpress : Create wp-config.php ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 0.47s
mariadb : Start MariaDB ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 0.46s
mariadb : Create WordPress DB user -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- 0.39s

PLAYBOOK RECAP *****************************************************************************************************************************************************************************************************
Playbook run took 0 days, 0 hours, 2 minutes, 9 seconds
$
```

