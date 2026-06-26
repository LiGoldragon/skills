# Skill — research library

## What it is and when to use it

When a task wants depth — primary-source philosophy, formal
mathematics, ontology engineering, classical domain texts — read the
actual sources. Training-data priors are not a substitute for reading
when the work is supposed to be deep.

The library lives at `~/primary/repos/library/` — the scholarly
foundation sema's ontology is built on: classical source texts
(Ptolemy, Valens, Parasara, Lilly), modern systematizers, category
theory (Mazzola, Spivak, Zalamea), correspondence systems, Vedic
philosophy, and the project's own work. It is a sibling resource, NOT
canonical workspace state — never move references to it inside
`~/primary/`.

Consult it when a foundational concept needs a real source, when
auditing/comparing formal systems, when designing an abstraction
inspired by a known framework (verify the framework's actual claims,
not a paraphrase), or when a report is meant to be deep. Skip it for
quick recall of well-known facts and for language/implementation
reference — use the language's own spec. Rule of thumb: if the task
asks for *depth* or *research*, read the library.

## Layout

```
~/primary/repos/library/
├── bibliography.md        — complete tiered bibliography with MD5 hashes
├── documentation-spec.md  — category-theoretic documentation framework
├── en|fr|de|el|la|sa/<author-name>/  — sources by language, one dir per author
```

Author directories are lowercase-hyphenated (`john-sowa`,
`david-spivak`). Filenames encode work + translator/edition
(`conceptual-structures.pdf`, `tetrabiblos-robbins-loeb.pdf`).
Binary files (PDF, EPUB, DJVU) are gitignored locally and tracked by
Anna's Archive MD5 hashes; the MD5 is the bridge back to the archive
if a machine loses the local file.

## Reading from the library

Read PDFs via the `Read` tool with `pages:` (max 20 pages per
request — batch long books; `pages` is mandatory for PDFs over 10
pages):

```
Read /home/li/primary/repos/library/en/john-sowa/conceptual-structures.pdf pages:1-15
```

`Read` does not handle EPUB or DJVU — convert first:

```sh
nix run nixpkgs#calibre -- ebook-convert input.epub output.pdf
nix run nixpkgs#djvulibre -- ddjvu -format=pdf input.djvu output.pdf
```

## Adding new books — the `annas` CLI

`annas` searches and downloads from Anna's Archive. It's a Go binary
on PATH via home-manager (`/home/li/.nix-profile/bin/annas`); if
`which annas` is empty, `find /nix/store -maxdepth 2 -name annas -type f`
and invoke by store path.

Always run it from the library directory so it finds `.env`. The CLI
emits a noisy startup `WARN` line and Go stack trace when `.env` is
missing; filter it out when you just want results:

```sh
cd ~/primary/repos/library && annas book-search "spivak ologs"
cd ~/primary/repos/library && annas book-search "category theory" 2>&1 \
  | grep -v "WARN\|Error loading\|cli.go\|main.go\|proc.go\|annas-mcp\|runtime.main\|StartCLI\|^github"
```

Config is two variables in the gitignored
`~/primary/repos/library/.env`:

```
ANNAS_SECRET_KEY=<API key from annas-archive.li>
ANNAS_DOWNLOAD_PATH=/home/li/primary/repos/library/<lang>/<author>/
```

The workspace runs unauthenticated: searches work without a key;
downloads may require one.

Search and download (results print as a table with MD5 hashes — the
MD5 is what `bibliography.md` records):

```sh
annas book-search "category theory for sciences"
annas article-search "10.1038/nature12345"
annas book-download <md5-hash> "category-theory-for-sciences.pdf"
annas article-download "10.1038/nature12345"
```

Filenames follow the lowercase-hyphenated work+translator convention.

After downloading, add an entry to `bibliography.md` in the right
tier/author section — files not in the bibliography drift and won't
be found by topic search:

```markdown
### Author Name — Work Title (date)
Short description of why this work is here.
- `en/author-name/` — Translator/edition: `<md5-hash>`
```

## Orientation by domain

Not exhaustive — see `bibliography.md`. For a foundational citation:

| Need | Reach for |
|---|---|
| Knowledge representation / ontology | Sowa; Guarino |
| Category-theoretic data modeling | Spivak; Mazzola; Zalamea |
| Meaning-of-meaning | Frege; Wittgenstein; Quine |
| Truth, model theory | Tarski |
| Categorial semantics of natural language | Aarne Ranta |
| Classical philosophy | Aristotle; Plato; Plotinus |
| Indian philosophy | Patanjali; Sankara; Nagarjuna; Bhartrhari |
| Astrology (sema's domain) | Ptolemy; Valens; Firmicus; Lilly |
| Linguistics | Halliday; Tesnière; Mel'čuk |
| Pattern languages | Christopher Alexander |
| Hermeticism / correspondence | Hermes Trismegistus; Crowley; Skinner |

## Citing in reports

Cite a library text with author + year + title + section, plus the
local path, so the next agent can verify by reading the same pages:

```markdown
- **Sowa, J. F. (1984).** *Conceptual Structures: Information
  Processing in Mind and Machine.* Addison-Wesley.
  Chapter 1 §1.4 (Intensions and Extensions).
  Local: `~/primary/repos/library/en/john-sowa/conceptual-structures.pdf`
```

If a claim paraphrases from memory without reading, say so. "Per my
prior knowledge of Frege's sense/reference distinction" is fair;
"Frege showed in §2 that…" must correspond to actual reading.

## Anti-patterns

- **Citing without reading.** Drive-by "as Sowa showed" with no page
  reference is the exact failure this skill prevents.
- **Training-data analogues for deep work.** Familiarity with a name
  is not knowledge of the named work's actual claims.
- **Downloading without updating `bibliography.md`.** Untracked files
  in author directories vanish from topic search.

## See also

- `~/primary/repos/library/bibliography.md` — the full curated index.
- `~/primary/skills/reporting.md` — report citation form.
