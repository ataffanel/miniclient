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
    && rm -rf /var/lib/apt/lists/*

RUN apt-get update && apt-get install -y \
    file \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo install cargo-appimage

# Install appimagetool
RUN curl -fsSL -o /usr/local/bin/appimagetool \
    https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage \
    && chmod +x /usr/local/bin/appimagetool

# appimagetool needs this in containers (no FUSE available)
ENV APPIMAGE_EXTRACT_AND_RUN=1

WORKDIR /src
