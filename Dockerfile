# Stage 1: Build the application
FROM rust:1-bookworm as builder

# Create a new empty shell project
RUN USER=root cargo new --bin visualize-api
WORKDIR /visualize-api

COPY ./ ./

# Build the application
RUN cargo build --release

# Stage 2: Setup the runtime environment
FROM debian:bookworm-slim as runtime

# Copy the binary and any other necessary files from the builder stage
COPY --from=builder /visualize-api/target/release/visualize-api /usr/local/bin/visualize-api

ENV API_URL="https://api.cloud.cbh.kth.se"
ENV ROCKET_PROFILE="release"

# Install necessary runtime dependencies
RUN apt update && apt upgrade -y && apt autoremove -y 
RUN apt install libssl-dev ca-certificates -y && rm -rf /var/lib/apt/lists/*

EXPOSE 8000

# Set the default command to run the binary
CMD ["visualize-api"]