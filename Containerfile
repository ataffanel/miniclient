FROM ubuntu:24.04

RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    pkg-config \
    libfontconfig1-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libxkbcommon-dev \
    libwayland-dev \
    libgl-dev \
    libudev-dev \
    qtbase5-dev \
    patchelf \
    file \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install linuxdeploy and its Qt plugin
RUN curl -fsSL -o /usr/local/bin/linuxdeploy \
    https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage \
    && chmod +x /usr/local/bin/linuxdeploy

RUN curl -fsSL -o /usr/local/bin/linuxdeploy-plugin-qt \
    https://github.com/linuxdeploy/linuxdeploy-plugin-qt/releases/download/continuous/linuxdeploy-plugin-qt-x86_64.AppImage \
    && chmod +x /usr/local/bin/linuxdeploy-plugin-qt

# AppImage tools need this in containers (no FUSE available)
ENV APPIMAGE_EXTRACT_AND_RUN=1

WORKDIR /src
