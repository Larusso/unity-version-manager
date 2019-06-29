FROM japaric/x86_64-unknown-linux-gnu:v0.1.4

RUN mkdir -p /home/ci/.cache
RUN mkdir -p /home/ci/.config
RUN mkdir -p /home/ci/.local/share

# Set the home directory to our app user's home.
ENV HOME=/home/ci
ENV RUST_BACKTRACE=1
ENV RUST_LOG='warning, uvm_core=trace'

RUN apt-get update && \
    apt-get install -y build-essential libssl-dev pkg-config openssl p7zip-full cpio

# Chown all the files to the app user.
RUN chmod -R 777 $HOME
