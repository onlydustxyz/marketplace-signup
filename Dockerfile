####################################################################################################
## Builder
####################################################################################################
FROM rust:1.61 AS builder

RUN update-ca-certificates

# Create unprivileged app user
RUN adduser --disabled-password od-app-user

WORKDIR /app

COPY ./ .

# CAUTION! please make sure the binary is named "service" by adding those 3 lines to Cargo.toml:
# [[bin]]
# name = "service"
# path = "src/main.rs"
RUN cargo build --release

####################################################################################################
## Final image
####################################################################################################
FROM gcr.io/distroless/cc
ARG SERVICE_NAME

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /app

# Copy our build
COPY --from=builder /app/target/release/* /app/

# Expose port
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=80
EXPOSE 80

# Add labels
LABEL "com.datadoghq.ad.logs"="[{""source"": ""deathnote"", ""service"": ""${SERVICE_NAME}""}]"

# Use app user
USER od-app-user:od-app-user

CMD ["/app/service"]
