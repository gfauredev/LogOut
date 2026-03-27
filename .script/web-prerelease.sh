mkdir --mode 775 --verbose dist
gh release download --pattern "web.tar.gz" --dir .
if [ -f "web.tar.gz" ]; then
  tar -xzvf web.tar.gz -C dist/
  if [ -d "dist/LogOut" ]; then
    chmod 775 dist/LogOut
  else
    echo "WARNING: No LogOut directory found in web archive, stopping now"
    exit 1
  fi
else
  echo "WARNING: No web archive found in latest release, stopping now"
  exit 1
fi
nix build .#preWeb --out-link preWeb
cp --dereference --recursive -v preWeb/LogOut/preview dist/LogOut/preview
