# Use the official Rust image as the base image
FROM rust:1.76

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Copy the .env file
COPY .env .env

# Build the application
RUN cargo build --release

# Run the application
CMD ["./target/release/posts-axum"]
