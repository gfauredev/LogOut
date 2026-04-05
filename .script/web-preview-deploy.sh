mkdir --mode 775 --verbose web
gh release download --pattern "web.tar.gz" --dir .
if [ -f "web.tar.gz" ]; then
  tar -xzvf web.tar.gz -C web/
  if [ -d "web/LogOut" ]; then
    chmod --recursive 775 web/LogOut
  else
    echo "WARNING: No LogOut directory found in web archive, stopping now"
    exit 1
  fi
else
  echo "WARNING: No web archive found in latest release, stopping now"
  exit 1
fi
nix build .#preWeb --out-link preWeb
cp --dereference --recursive -v preWeb/LogOut/preview web/LogOut/preview
