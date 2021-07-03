#!/usr/bin/env bash 

set -x 
set -eof pipefail

su - postgres <<'EOSU'
psql -c "create database RHD with encoding 'unicode';"
psql -c "create user RHD_admin with encrypted password 'password';"
psql -c "grant all privileges on database RHD to RHD_admin;"
EOSU

