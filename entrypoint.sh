#!/bin/bash
set -e

# CORRECTION : Utiliser le même dossier que le WORKDIR du Dockerfile
ls /usr/src/message
cd /usr/src/message

# Lancer cargo watch
exec cargo watch --poll -w src -i target -x run
