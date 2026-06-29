# Skill — secrets

## Rules

Treat secret bytes as transient hazardous material. Do not put plaintext secrets in chat, commits, logs, traces, generated outputs, documentation, shell history, process arguments, or the Nix store.

Prefer blind movement from the secret store to the consuming tool. Pipe plaintext directly to the sink; inspect only when the task truly requires human-readable verification.

Public keys, recipients, key names, and ciphertext are not secret by themselves. The secret value is.

## Handling

Use the workspace secret store for interactive secrets and encrypted deployment files for host activation. Commit only ciphertext and declarations, never decrypted values.

For a program that needs an environment variable, wrap the program so the secret is fetched at execution time. Do not bake the value into a package, unit file, wrapper text, or flake output.

When minting a secret, generate with a CSPRNG, insert directly into the secret store, and verify by exit status or entry presence rather than printing the value. Refuse accidental overwrite unless the task is explicitly rotating the secret.

When bridging to encrypted deployment material, stream secret store output directly into the encryption command. Verify encryption by ciphertext markers and expected public recipients.

If a task needs the value displayed or copied, keep it out of durable outputs and state exactly where it was consumed.
