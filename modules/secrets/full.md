# Skill — secrets

## Agents may inspect tokens when the task requires it

Authentication tokens and other secret values may reach the agent's eyes
when plaintext inspection is needed for an authorized workspace task.
Visible plaintext is transient working material. A secret must not appear
in any durable or broadly visible surface:

- a log line, report, chat message, or commit message;
- a shell trace (`set -x` while a secret variable is live);
- the nix store, a test fixture, or a checked-in plaintext file.

Keep exposure narrow:

- **Prefer pipe source to sink.** Move a secret by connecting the
  producer's stdout to the consumer's stdin when blind handling is
  enough. The value lives only in the pipe buffer and the two processes'
  memory.
- **Inspect only when useful.** If seeing the token is the simplest safe
  way to debug, verify, copy, or configure it, do so deliberately and
  keep it out of durable surfaces.
- **Keep `argv` clean when possible.** A command's `argv` is visible to
  any `ps` on the box. Prefer stdin, an environment variable scoped to
  the one command, or a protected runtime file.
- **Verify blind when enough.** Confirm success by exit code, byte
  length (`... | wc -c`), entry name (`gopass ls | grep -F <name>`), or
  ciphertext markers (`grep ENC\[ <file>.sops`) when the value itself
  does not need inspection.
- **Public keys are public.** Age recipients, nix cache public keys, and
  ssh public keys may appear in `argv` and output freely. Only the
  secret bytes need the transient-material discipline.

## Two layers: gopass for the session, sops-nix for the cluster

- **gopass** encrypts secrets within the user session — git-backed,
  per-user, decrypted on demand through the user's own key. It is the
  human-controlled source of truth for interactive and development use.
- **sops-nix** carries secrets to cluster hosts. The secret is
  encrypted at rest in the repository (only ciphertext is committed)
  and decrypted **only on the target host at activation**, into a
  runtime tmpfs at `/run/secrets/<name>`.

These compose: mint a secret into gopass, then bridge it into a sops
file for deployment. The *plaintext* never enters the nix store; the
sops file holds only ciphertext, and decryption happens at activation
outside the store.

## gopass: wrapping environment variables at the daemon-wrapper layer

A binary that needs a secret in an environment variable is **wrapped**
so the secret is fetched fresh at exec time, never baked into the
package or the systemd unit:

```nix
pkgs.symlinkJoin {
  name = "flarectl-wrapped";
  paths = [ pkgs.flarectl ];
  nativeBuildInputs = [ pkgs.makeWrapper ];
  postBuild = ''
    wrapProgram $out/bin/flarectl \
      --run 'export CF_API_TOKEN=$(${pkgs.gopass}/bin/gopass show -o cloudflare/api-token)'
  '';
}
```

The secret is read at each invocation (rotation needs no rebuild),
never stored in the store path, never written to the unit file.
Command substitution `$(...)` strips the trailing newline, so the
exported value is the clean secret.

**Path conventions.** Provider-scoped for an external provider's single
global credential (`cloudflare/api-token`); zone-scoped for a local
service in a cluster zone
(`goldragon.criome/local-llm-api-token`) so the path survives the
service moving between hosts.

## Minting a secret

Generate with a CSPRNG and pipe straight into the store unless the task
requires reading the generated value:

```sh
token=$(head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n')   # 256-bit hex
printf '%s\n' "$token" | gopass insert -f <path> >/dev/null
```

Confirm by exit code and `gopass ls | grep -F <name>`. Make minting
idempotent: refuse to overwrite an existing entry unless an explicit
`--rotate` flag is given, because rotation forces every consumer to
re-read.

## sops-nix: how cluster secrets decrypt on the host

- **Host key.** Decryption uses the host's SSH ed25519 key converted to
  age (`sops.age.sshKeyPaths = [ "/etc/ssh/ssh_host_ed25519_key" ]`).
  No separate per-host age key file to manage.
- **Recipient.** Encrypt to the host's age recipient, derived from its
  ssh public key: `echo 'ssh-ed25519 <body>' | ssh-to-age`. Cross-check
  the derived value against the recipient on an existing working secret
  before trusting it — encrypting to the wrong key fails silently (the
  host simply cannot decrypt).
- **File shape.** The binary store: a JSON file
  `{"data":"ENC[AES256_GCM,...]","sops":{"age":[{recipient,enc}]}}`,
  consumed with `format = "binary"`.
- **Declaration.** In a host module:

  ```nix
  sops.secrets.<name> = {
    format = "binary";
    sopsFile = <the .sops file>;
    owner = "<service-user>";
    mode = "0400";
    restartUnits = [ "<service>.service" ];
  };
  ```

  The service reads `config.sops.secrets.<name>.path`
  (`/run/secrets/<name>`). Rotation is handled by `restartUnits`.
- **Deploy wiring is per-cluster.** How the encrypted file becomes a
  flake input the host config references is the deploy tooling's job;
  consult the cluster repo's own docs.

## The blind bridge: gopass to sops-nix

Move a secret from gopass into a sops file without displaying it when
blind handling is enough:

```sh
gopass show -o <gopass-path> \
  | sops --encrypt --age <recipient-public-key> \
      --input-type binary --output-type binary /dev/stdin \
  > <file>.sops
```

The plaintext flows gopass → pipe → sops; it never touches a terminal
or `argv`. The recipient is a public key, safe on the command line.
`gopass show` triggers decryption (the pinentry prompt is the human
unlocking their own store). The agent may keep this bridge blind or may
inspect the value if the task requires it; the durable output remains
ciphertext. Verify blind when enough: `grep ENC\[ <file>.sops` for
encryption and `grep -oE 'age1[a-z0-9]+' <file>.sops` for the recipient
set.

## See also

- `skills/system-operator.md` — deploy surface; keys come from gopass
  at the daemon-wrapper layer.
- `skills/nix-discipline.md` — services as NixOS modules; store-path and
  secret-state hygiene.
