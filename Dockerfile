# Stage 1: Build the application
# Use the official Rust image as a base
FROM rust:1-bookworm as builder

# Create a new empty shell project
RUN USER=root cargo new --bin visualize-api
WORKDIR /visualize-api

COPY ./Cargo.toml ./
COPY ./src ./src

# Build the application
RUN cargo build --release

# Stage 2: Setup the runtime environment
# Use a Debian or Ubuntu image that is compatible with the GLIBC version used in the builder stage
FROM debian:bookworm-slim as runtime

# Copy the binary and any other necessary files from the builder stage
COPY --from=builder /visualize-api/target/release/visualize-api /usr/local/bin/visualize-api

ENV API_URL="https://api.cloud.cbh.kth.se"
ENV ROCKET_ENV=production
ENV ROCKET_PORT=8000

RUN apt update && apt install -y libssl-dev 

# Set the default command to run the binary
CMD ["visualize-api"]