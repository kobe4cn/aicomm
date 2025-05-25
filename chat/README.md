# Rust 语言

# Generate ed25519 privkey
openssl genpkey -algorithm ed25519 -out privkey.pem
# export its pubkey
openssl pkey -in privkey.pem -pubout -out pubkey.pem

# Generate RSA privkey
openssl genrsa -des3 -out private.pem 2048
# export its pubkey
openssl rsa -in private.pem -outform PEM -pubout -out public.pem
