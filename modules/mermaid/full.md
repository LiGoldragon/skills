# Skill — Mermaid syntax

## Rules

Target the strictest renderer that will publish the diagram. A successful render
in a permissive preview is not proof that the target surface accepts the block.

Keep each diagram to one question: data flow, control flow, type lineage,
failure mode, or topology. Use several small diagrams instead of one system map.

Aim for four to eight nodes. Split before edge crossings, sideways scrolling, or
subgraphs become necessary to rescue readability.

Use simple ASCII node identifiers. Put visible text in bracket labels:
`node_id["short label"]`. Do not use quoted strings as node IDs.

Quote node labels when they contain spaces, punctuation, slashes, or hyphens.
Keep labels short: a node is a noun phrase, not a paragraph.

Keep locators, long identifiers, bead IDs, and citations out of nodes. If the
identifier matters, put a short label in the graph and map it in nearby prose or
a table.

Edge labels go in pipes: `a -->|feeds| b`. Do not write quoted strings where an
edge label belongs.

Edge labels are plain prose. Avoid sigils, punctuation-heavy notation, and
Unicode arrows inside labels; put notation in a node label or surrounding prose.

Avoid reserved node IDs: graph, flowchart, subgraph, end, class, classDef, style,
link, linkStyle, note, click, direction. Suffix IDs by role when in doubt.

Subgraphs use an identifier plus a plain bracket title:
`subgraph storage_group [Storage boundary]`. Avoid quoted titles, punctuation,
and subgraph-local direction.

Sequence and state diagrams reject semicolon-heavy text in older renderers. Use
commas, "and", or separate messages.

Put spaces around sequence arrows, especially arrow forms containing letters or
symbols.

Do not depend on manual line breaks, HTML breaks, or renderer wrapping. If a
label needs wrapping, shorten it or move the detail out of the diagram.

Inspect the diagram in the target surface. If labels clip, truncate, crowd, or
force horizontal scrolling, rewrite before publishing.
