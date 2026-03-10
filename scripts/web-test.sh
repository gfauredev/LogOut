#!/bin/sh
set -e
nix build .#pages -o dist/LogOut/
nix develop -c python3 -m http.server -d dist/ 8080 &
sleep 2
