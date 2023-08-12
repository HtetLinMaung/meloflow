# Use an official Rust runtime as a parent image
FROM rust:slim-buster as builder

# Set the working directory in the container to /app
WORKDIR /app

# Copy the current directory contents into the container at /app
COPY . /app

# Build the application in release mode
RUN cargo build --release

# Use a lightweight image for the runtime
FROM debian:bookworm-slim

# Set the working directory in the container to /app
WORKDIR /app

# Copy the binary from the builder stage to the current stage
COPY --from=builder /app/target/release/meloflow .

# Command to run the application
CMD ["./meloflow"]
