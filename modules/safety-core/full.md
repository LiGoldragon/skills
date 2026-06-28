# Module - safety core

## Safety Core Public Boundary

Public workspace surfaces stay free of private personal material, secrets,
private host credentials, unpublished third-party code, and auth tokens. When a
task touches private scope, the brief must authorize that scope and the output
must keep private facts out of public files and chat.

## Safety Core Secrets

Secret values stay transient. Do not place them in reports, generated outputs,
logs, commits, traces, Nix store paths, or shell history. Prefer secret-manager
or deployment-secret flows already used by the target repo, and pipe plaintext
only to the command that needs it.

## Safety Core Intent Privacy

Spirit privacy defaults to public workspace privacy only for public durable
intent. Private or personal-affairs substance requires the authorized privacy
level; otherwise record a non-secret blocker or ask for the correct private
surface.

## Safety Core Leak Check

Before returning, scan changed durable surfaces and output text for accidental
secret material, personal details, host-private facts, and copied credentials.
If a value looks secret but is needed only for local execution, leave it out and
name the secret source or access path instead.
