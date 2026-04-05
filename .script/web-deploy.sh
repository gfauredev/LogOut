mkdir --mode 775 --verbose web
nix build .#web --out-link relWeb
cp --dereference --recursive -v relWeb/LogOut web/LogOut
chmod --recursive 775 web # Files from Nix store come read-only
tar --create --directory=web --file=web.tar.gz --gzip --verbose LogOut
nix build .#preWeb --out-link preWeb # Set preview to this Release
cp --dereference --recursive -v preWeb/LogOut web/LogOut/preview
