FROM clux/muslrust:1.83.0-stable as builder

RUN apt-get update && apt-get install -y cmake libprotobuf-dev protobuf-compiler

# Make a new group and user so we don't run as root.
ARG UID=1000
ARG GID=1000
RUN addgroup --system -u $UID appgroup && adduser --system -u $UID appuser --ingroup appgroup

WORKDIR /volume
COPY . .

# Build the binary.
RUN cargo build --release

FROM scratch
# Import the user and group files from the builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

# Copy our static executable.
COPY --from=builder /volume/target/x86_64-unknown-linux-musl/release/mh-events2pulsar .

# Use an unprivileged user.
USER appuser:appgroup

# Run the binary
ENTRYPOINT [ "/mh-events2pulsar" ]