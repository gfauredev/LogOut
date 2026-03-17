FROM nixos/nix:latest
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf
WORKDIR /workspace
CMD ["nix", "develop", "--command", "bash"]
