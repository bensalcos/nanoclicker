FROM ubuntu:24.04@sha256:68ee22004a9109b44973313b02ff7b1de250f1de0f5ecb628d73f579aaff33ac
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    xvfb xauth libgl1-mesa-dri libglx-mesa0 libegl1 libxkbcommon0 libxkbcommon-x11-0 libx11-6 libxi6 libxrandr2 \
    libxcursor1 libxinerama1 libvulkan1 mesa-vulkan-drivers dbus-x11 \
    && rm -rf /var/lib/apt/lists/*
# Test-only image: no application, source tree or credentials are copied in.
USER 65534:65534
