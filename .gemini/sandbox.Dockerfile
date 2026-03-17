FROM nixos/nix:latest
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf
RUN echo 'if [ -z "$HAS_ENTERED_NIX_DEVELOP" ]; then \
            export HAS_ENTERED_NIX_DEVELOP=1; \
            exec nix develop; \
          fi' >> /etc/bash.bashrc && \
    echo 'if [ -z "$HAS_ENTERED_NIX_DEVELOP" ]; then \
            export HAS_ENTERED_NIX_DEVELOP=1; \
            exec nix develop; \
          fi' >> /root/.bashrc
WORKDIR /workspace
CMD ["sleep", "infinity"]
