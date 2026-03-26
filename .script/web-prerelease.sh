mkdir --parents --verbose dist/LogOut
# Download current stable Pages deployment to preserve it during pre-release
if ARTIFACT_URL=$(gh api "repos/{owner}/{repo}/actions/artifacts?name=github-pages&per_page=100" --jq '[.artifacts[] | select(.expired == false)] | sort_by(.created_at) | last | .archive_download_url // empty'); then
  if [ -n "$ARTIFACT_URL" ]; then
    gh api "$ARTIFACT_URL" >release.zip
    unzip release.zip -d release/
    if [ -f release/artifact.tar ]; then
      tar -xf release/artifact.tar -C dist/LogOut/
    else
      echo "WARNING: artifact.tar not found, stopping now to not destroy prod"
      exit 1
    fi
  else
    echo "WARNING: No previous stable Pages deployment found, stopping now"
    exit 1
  fi
else
  echo "WARNING: Could not fetch artifacts list, stopping now"
  exit 1
fi
nix build .#preWeb --out-link preWeb
cp --dereference --recursive -v preWeb/LogOut/preview dist/LogOut/preview
