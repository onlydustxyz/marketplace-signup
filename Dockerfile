####################################################################################################
## Builder
####################################################################################################
FROM rust:1.61 AS builder

RUN update-ca-certificates

# Create unprivileged app user
RUN adduser --disabled-password od-app-user

WORKDIR /app

COPY ./ .

RUN cargo build --release

####################################################################################################
## Final image
####################################################################################################
FROM gcr.io/distroless/cc

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /app

# Copy our build
COPY --from=builder /app/target/release/* /app/

# Use app user
USER od-app-user:od-app-user

CMD ["/app/od-badge-signup"]