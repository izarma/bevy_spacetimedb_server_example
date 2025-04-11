client-run:
	cargo run --package client-bevy

# --- Server Targets ---

server-build:
	spacetime build -p server

server-publish:
	spacetime publish -p server --server local game1

server-publish-delete:
	spacetime publish -p server --server local game1 --delete-data -y

# Generate code for the bevy client.
autogen:
	spacetime generate -p server -y --lang rust --out-dir client-bevy/src/stdb