# Use the latest LTS version of Ubuntu
FROM ubuntu:22.04

ENV XDG_RUNTIME_DIR=/tmp/xdg

# Set environment variable for offscreen rendering
ENV WGPU_FORCE_OFFSCREEN=true
ENV WGPU_NATIVE_VERSION=v0.14.2.3

# Set environment variables for Rust
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV RUST_BACKTRACE=1
ENV RUSTFLAGS="-A unused_mut"

# Update and install necessary dependencies
RUN apt-get update -y -qq
RUN apt-get install -y software-properties-common curl build-essential cmake clang git
RUN add-apt-repository -y ppa:deadsnakes/ppa
RUN add-apt-repository -y ppa:oibaf/graphics-drivers
RUN apt-get update -y -qq
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y python3.9 python3.9-distutils
RUN apt-get install -y libegl1-mesa libgl1-mesa-dri libxcb-xfixes0-dev mesa-vulkan-drivers
RUN apt-get clean
RUN rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Upgrade pip and install Python dependencies
RUN curl https://bootstrap.pypa.io/get-pip.py | python3.9
RUN python3.9 -m pip install --upgrade pip
RUN python3.9 -m pip install wgpu numpy pathlib pillow requests pytest

# Build wgpu-native from source (pip won't download it automatically in a container)
RUN git clone --depth 1 --branch v0.14.2.3 https://github.com/gfx-rs/wgpu-native.git && \
    cd wgpu-native && \
    git submodule update --init && \
    make lib-native-release && \
    cp -r ./target/release/* /usr/local/lib/python3.9/dist-packages/wgpu/resources

# Copy the built library to the Python library directory
RUN cd /usr/local/lib/python3.9/dist-packages/wgpu/resources && \
    cp libwgpu_native.so libwgpu_native-release.so && \
    cp libwgpu_native.a libwgpu_native-release.a

# Copy your Python project into the container
COPY ./ /usr/src/myapp
WORKDIR /usr/src/myapp

# CMD to run your Python script or any other command
CMD ["bash"]
