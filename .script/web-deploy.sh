mkdir --mode 775 --verbose web
nix build .#web --out-link relWeb
cp --dereference --recursive -v relWeb/LogOut web/LogOut
tar -czvf web.tar.gz -C web LogOut
# Copy Release in Preview to avoid 404s
cp --dereference --recursive -v relWeb/LogOut web/LogOut/preview
