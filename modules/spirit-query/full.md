# Skill — spirit query

## Query Rules

Use `spirit` for read-only intent queries before judgment. Query relevant public intent early when orchestrating, auditing, scouting, translating, designing, editing doctrine, or deciding how a brief should map to durable guidance. Purely mechanical workers may skip this when the brief already supplies the needed intent context.

Use domain-first `PublicRecords` as the normal query path. Start with the narrowest matching domain or subtree, then widen only when the result lacks enough intent evidence. Use `Lookup` when the brief or a previous query gives a known record identifier.

Public reads are the default. Private reads need explicit prompt authorization for that privacy scope, and private content stays out of public chat, reports, commits, and generated doctrine.

## Query Shapes

The CLI takes exactly one argument: inline NOTA when the argument starts with `(`, or a NOTA file otherwise. It replies on stdout with typed NOTA and returns nonzero on transport, parse, or daemon errors.

List public records in the narrowest relevant domain first:

```sh
spirit "(PublicRecords ((Full [(Technology (Software (Intelligence AgentSystems)))]) None))"
```

Lookup a known record identifier:

```sh
spirit "(Lookup <record-id>)"
```

Treat `(Error [record not found])` and `(Error [no matching record])` as negative evidence, not tool failure. Treat validation rejection, parse failure, daemon failure, or unexpected wire shape as a blocker for intent-grounded judgment.

## Domain List

Use these current Spirit domains and subdomains when forming `PublicRecords` scopes:

- `All`
- `Health`: `Body`, `Mind`, `Nutrition`, `Exercise`, `Sleep`, `Medicine`, `Disease`, `Medication`, `Therapy`, `Reproduction`, `Sexuality`, `Aging`, `Disability`, `Addiction`, `Dentistry`, `Senses`, `Pain`, `Prevention`, `FirstAid`, `Rehabilitation`
- `Food`: `Cooking`, `Diet`, `Recipe`, `Baking`, `Preservation`, `Fermentation`, `Beverage`, `Entertaining`, `Foraging`, `Fasting`, `Dining`
- `Home`: `Housing`, `Maintenance`, `Renovation`, `Furnishing`, `Cleaning`, `Tidying`, `Relocation`, `Realty`, `Property`, `Utilities`, `Locksmithing`, `Appliances`
- `Finance`: `Budgeting`, `Saving`, `Spending`, `Debt`, `Credit`, `Investing`, `Retirement`, `Tax`, `Insurance`, `Income`, `Banking`, `Charity`, `Planning`, `Accounting`
- `Work`: `Career`, `JobSearch`, `Workplace`, `Vocation`, `Leadership`, `Entrepreneurship`, `Employment`, `Compensation`, `Scheduling`, `Unemployment`, `Freelancing`, `Teamwork`, `Productivity`, `Project`
- `Craft`: `Electronics`, `Construction`, `Carpentry`, `Metalworking`, `Sewing`, `Manufacturing`, `Repair`, `Engineering`, `Handicraft`, `Invention`
- `Knowledge`: `Mathematics`, `Logic`, `Physics`, `Chemistry`, `Biology`, `Astronomy`, `Geology`, `Computing`, `Physiology`, `Statistics`, `Research`, `History`, `Linguistics`, `Philosophy`, `Economics`, `Cognition`, `Taxonomy`
- `Education`: `Studying`, `Teaching`, `Schooling`, `Skill`, `Reading`, `Memorization`, `Pedagogy`, `Mentoring`, `Autodidacticism`, `Credential`
- `Language`: `Writing`, `Rhetoric`, `Translation`, `Grammar`, `Conversation`, `Correspondence`, `Listening`, `Oratory`, `Editing`, `Terminology`, `Notation`
- `Art`: `Fiction`, `Poetry`, `Music`, `Painting`, `Photography`, `Film`, `Theater`, `Dance`, `Design`, `Sculpture`, `Creativity`, `Storytelling`, `Publishing`
- `Kinship`: `Friendship`, `Romance`, `Marriage`, `Family`, `Parenting`, `Relatives`, `Reconciliation`, `Boundaries`, `Intimacy`, `Rapport`, `Caregiving`, `Grief`, `Belonging`
- `Selfhood`: `Growth`, `Introspection`, `Discipline`, `Emotion`, `Virtue`, `Motivation`, `Confidence`, `Identity`, `Purpose`, `Decision`, `Temperament`, `Wellbeing`, `Composure`
- `Spirituality`: `Worship`, `Prayer`, `Meditation`, `Ritual`, `Faith`, `Theology`, `Contemplation`, `Pilgrimage`, `Scripture`, `Ethics`, `Mortality`, `Transcendence`, `Asceticism`, `Wisdom`
- `Governance`: `Politics`, `Government`, `Administration`, `Citizenship`, `Elections`, `Activism`, `Policy`, `Diplomacy`, `Movements`, `Organizing`, `Services`, `Naturalization`, `War`
- `Law`: `Rights`, `Contract`, `Title`, `Crime`, `Litigation`, `Compliance`, `Custody`, `Liability`, `Procedure`, `Justice`, `Policing`, `Arbitration`
- `Community`: `Neighborliness`, `Volunteering`, `Solidarity`, `Membership`, `Gatherings`, `Reputation`, `Service`, `Hospitality`, `Institutions`
- `Nature`: `Agriculture`, `Gardening`, `Horticulture`, `Husbandry`, `Pets`, `Forestry`, `Fishing`, `Hunting`, `Conservation`, `Weather`, `Wilderness`, `Sustainability`, `Resources`, `Stewardship`
- `Travel`: `Itinerary`, `Destination`, `Transportation`, `Driving`, `Navigation`, `Commuting`, `Logistics`, `Migration`, `Tourism`, `Transit`, `Cycling`
- `Commerce`: `Selling`, `Buying`, `Marketing`, `Retail`, `Sourcing`, `Trade`, `Support`, `Pricing`, `Negotiation`, `Assets`, `Market`
- `Leisure`: `Recreation`, `Sport`, `Games`, `Hobby`, `Entertainment`, `Collecting`, `Outdoors`, `Play`, `Relaxation`, `Celebration`, `Fandom`
- `Appearance`: `Clothing`, `Grooming`, `Style`, `Cosmetics`, `Etiquette`, `Comportment`
- `Safety`: `Protection`, `Preparedness`, `Risk`, `Cybersecurity`, `Privacy`, `Disaster`, `Military`, `Deterrence`
- `Information`: `Curation`, `RecordKeeping`, `Documentation`, `News`, `Broadcasting`, `Archives`, `Database`, `Retrieval`, `Classification`
- `Technology`: `Hardware(All, Networking)`; `Software(Programming(All, TypeSystems, Compilation, Parsing, Grammars, CodeGeneration, Metaprogramming, Macros, DomainSpecificLanguages), Theory, Systems(All, SystemsProgramming, Concurrency), Distributed(All, ProtocolDesign, EventDrivenArchitecture), Data(All, Persistence, Serialization, Formats, Modeling, SchemaEvolution, Migration), Intelligence(All, AgentSystems), Security(All, Cryptography, Authentication, Authorization, SecretsManagement, Privacy), Quality(All, Testing), Operations(All, BuildSystem, ReleaseEngineering, DependencyManagement, Deployment, ConfigurationManagement), Observability(All, Tracing), Surfaces(All, Visualization, CommandLineInterfaces), Engineering(All, Architecture, Design, ApplicationProgrammingInterfaces, Documentation, VersionControl, DevelopmentProcess, Management, Modularity))`

## Evidence

Report the query class, relevant record identifiers, and the conclusion needed for the task. Explain a Spirit identifier on first mention when it matters. Summarize record lists instead of pasting irrelevant hashes.

## Source Maintenance Notes

Refresh the generated-visible domain list from `signal-domain/schema/domain.schema`, then validate deployed help such as `spirit '(Help Domain)'` and relevant subtree help commands. Keep non-normal query interfaces out of generated guidance unless accepted direction makes them normal.
