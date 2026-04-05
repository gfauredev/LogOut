mkdir --mode 775 --verbose web
nix build .#web --out-link relWeb
cp --dereference --recursive -v relWeb/LogOut web/LogOut
tar -czvf web.tar.gz -C web LogOut
chmod --recursive 775 web # Files from Nix store come read-only
# Copy Release in Preview to avoid 404s
cp --dereference --recursive -v relWeb/LogOut web/LogOut/preview
