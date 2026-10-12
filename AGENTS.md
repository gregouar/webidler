# AGENTS.md

## Project intent

This repository contains **webidler**, a browser-based idle action RPG written in Rust. It takes inspiration from the depth of games such as Path of Exile: meaningful itemization, build experimentation, layered progression, and long-term optimization. It must remain its own game.

Gameplay design is owner-directed. Agents must not invent, rebalance, or expand the game on their own.

### Protected gameplay and content scope

- Do not change balance values, formulas, drop rates, rarity weights, affix ranges, enemy strength, encounter pacing, resource costs, rewards, progression speed, unlock requirements, or automation behavior unless the user explicitly requests that exact kind of change.
- Do not add or rewrite items, skills, passives, affixes, monsters, areas, encounters, quests, pets, lore, names, descriptions, or other game content without clear user-provided direction.
- A request to fix, refactor, optimize, migrate, restyle, or improve UI does not authorize gameplay or content changes. Preserve observable gameplay behavior and existing data values while completing such work.
- Do not make incidental "balance fixes," fill perceived content gaps, normalize unusual values, or replace deliberate mechanics based on personal judgment.
- If a requested feature cannot be completed without an unspecified design, balance value, reward, or content decision, implement the neutral/non-gameplay portion when possible and ask the user for that decision rather than guessing.
- When gameplay or content work is explicitly authorized, keep the change limited to the supplied requirements. Flag consequential interactions or missing parameters before broadening its scope.
- Any authorized new creative content must use original names, lore, art direction, skills, items, encounters, and progression concepts. Do not copy Path of Exile text, assets, characters, locations, passive-tree layouts, exact balance values, or proprietary terminology.

These instructions apply to the whole repository. More specific `AGENTS.md` files, if added later, override this file for their subtree.

## Repository map

- `backend/`: Axum HTTP/WebSocket server, authoritative game simulation, persistence, authentication, and integrations.
- `frontend/`: Leptos client compiled to WebAssembly with Trunk. It renders server state and sends player intent.
- `shared/`: Domain models, computations, HTTP types, and gameplay WebSocket messages shared by native and WASM targets.
- `backend_chat/`, `shared_chat/`: Chat service and its shared protocol.
- `backend_shared/`: Native-only helpers shared by backend services.
- `data/`: Manifest-driven JSON definitions for areas, monsters, items, affixes, loot, passives, skills, statuses, pets, and other game content.
- `migrations/sqlite/`, `migrations/postgres/`: Database migrations for both supported database engines.
- `tools/`: Leptos-based internal content and UI tools.
- `assets/`: Source/static game art consumed by the web builds.
- `docs/`: Design notes and idea documents. These are references, not necessarily current executable behavior.

## Architectural boundaries

### The server is authoritative

Combat results, rewards, inventory mutations, purchases, progression, cooldowns, and other cheat-sensitive decisions belong in `backend`. The frontend sends intent through `shared::messages::client::ClientMessage` and displays state from `shared::messages::server::ServerMessage`.

Never trust client-provided indexes, identifiers, quantities, ownership claims, or state. Validate bounds, existence, ownership, affordability, and current game phase on the server. Invalid input must fail safely without panicking or partially mutating state.

Do not implement a gameplay rule independently in both frontend and backend. Put portable data types and pure computations in `shared`; put authoritative mutation in `backend`; keep presentation-only derivations in `frontend`.

### Shared code must remain cross-target

`shared` is compiled for native Rust and `wasm32-unknown-unknown`. Avoid native-only APIs in common modules. Gate genuinely target-specific dependencies or implementations with `cfg`, following the existing compression code and `Cargo.toml` target dependency sections.

Protocol changes are vertical changes. When modifying a client or server message, update all constructors, match arms, serialization expectations, backend handlers, and frontend consumers together. Prefer additive, backward-tolerant changes when deployed clients and servers may briefly run different versions.

### Simulation changes must preserve tick semantics

The main loop in `backend/src/game/game_orchestrator.rs` has an intentional order: refresh dirty specs, control entities, resolve events, update entities, and advance elapsed time. Before moving a step or introducing side effects, trace how dirty flags, queued events, chained effects, death/respawn, waves, and synchronization depend on that order.

- Use `Duration` for elapsed/cooldown time and avoid frame-count assumptions.
- Use saturating or checked arithmetic where player input or long-running idle state can reach a boundary.
- Route random gameplay outcomes through the existing game RNG abstractions. Do not use UI randomness for authoritative results.
- Recompute derived stats through the existing updater/sync flow; do not patch cached derived values in unrelated controllers.
- Keep per-tick work bounded. Avoid database or network calls inside hot simulation loops.

## Data-driven game content

Treat `data/` as owner-authored game design, not as a backlog for autonomous improvement. Read it to understand dependencies and validate implementation work, but modify it only when the user clearly requests the specific content or balance change. Purely structural edits such as schema migrations must preserve all gameplay meaning and numerical values unless directed otherwise.

Follow the neighboring JSON schema and naming style in `data/`. Resource keys are stable identifiers: use lowercase `snake_case`, keep them unique, and do not rename a shipped key casually because saves and cross-resource references may depend on it.

Every content file must be reachable through its folder `.manifest.json`, and each enabled content folder must remain listed in `data/.manifest.json`. When adding or changing a reference, verify that the referenced item, skill, status, monster, area, loot table, icon, or passive exists.

Keep player-visible copy concise and original. Preserve exact enum spelling and serde representation expected by the Rust types. Use JSON numbers deliberately: check percentage versus multiplier semantics, local versus global scope, min/max ranges, unlock level, rarity, and whether quality applies.

A content change is not complete merely because the JSON parses. Check its progression position, drop source, UI display, asset path, and interactions with modifiers, triggers, conditions, and automation.

## Persistence and migrations

The backend supports SQLite by default and PostgreSQL in production/CI.

- Add a new timestamped migration to both `migrations/sqlite/` and `migrations/postgres/` unless a change is genuinely engine-specific and documented.
- Express the same schema and data transition in each dialect; account for their different auto-increment, type, and alter-table syntax.
- Never edit an already-deployed migration to change history. Add a forward migration.
- Keep database reads/writes in `backend/src/db/`; do not expose SQL details to game systems or UI components.
- Treat save compatibility as a feature. New fields in serialized long-lived state should have a migration or a safe serde/default path.
- Never commit secrets or real user data. Do not print JWTs, email addresses, password material, encryption keys, or full private save payloads in logs.

## Frontend conventions

Use existing Leptos components and signals before introducing a new state pattern. Pages belong in `frontend/src/components/pages`, reusable primitives in `components/ui`, shared game/account views in `components/shared`, live-run UI in `components/game`, and town-specific composition in `components/town`.

### Reuse before creating

Before writing markup for a new control or panel, search `frontend/src/components/ui` and `frontend/src/components/shared`, then inspect a nearby screen with similar behavior. Prefer composing or extending an existing component over recreating its appearance with local Tailwind classes.

Use the established components as the default building blocks:

- `MenuButton`, `MenuButtonRed`, `FancyButton`, `Toggle`, `TabButton`, and `CloseButton` for actions;
- `Card`, `MenuCard`, `CardHeader`, `CardTitle`, `CardInset`, and `CardInsetTitle` for surfaces and hierarchy;
- `MenuPanel` for overlays/panels and `ConfirmationModal`/`ConfirmContext` for destructive confirmation;
- `Input`, `ValidatedInput`, `Checkbox`, `DropdownMenu`, and `SearchableDropdownMenu` for form controls;
- `MenuListRow`, `Separator`, and `TitleSeparator` for lists and section structure;
- `DynamicTooltipTarget`, shared typed tooltips, `Toaster`, and `LoadingScreen` for feedback;
- `Number`, the formatting functions in `ui/number.rs`, and the existing progress bars for numerical state;
- shared `ItemCard`, inventory/bag, skill, passive, pet, resource, achievement, and leaderboard components for domain UI.

If an existing component is close but insufficient, add a small optional prop or variant when that produces a coherent reusable API and does not destabilize existing callers. Create a new primitive only when the interaction or visual role is genuinely distinct. Put generally reusable primitives in `components/ui`; put reusable game-domain presentation in `components/shared`. Do not duplicate tooltip formatting, rarity/damage colors, number formatting, modal shells, button chrome, card chrome, or resource icons in feature modules.

### Visual coherence

Match the established dark fantasy interface rather than inventing a separate style per feature. The current visual language uses near-black/zinc stone surfaces, muted brass/amber borders and accents, restrained red for danger, `Spectral`-style body typography, `Cinzel` through `font-display` for display text, clipped/octagonal panel shapes, and compact information-dense layouts.

- Copy spacing, sizing, breakpoint, and state patterns from the closest existing component. Prefer the shared component's props and classes over repeating its full class list at the call site.
- Preserve the semantic color language already used for item rarity, damage types, resources, success, warning, danger, disabled, selected, and hover states. Search for the relevant formatter/component instead of choosing a new hex value.
- Use `CardTitle`, separators, and the existing typography classes to maintain hierarchy. Do not introduce a new font, shadow language, border treatment, or corner shape for an isolated screen.
- Respect `SettingsContext`, `GraphicsQuality`, texture/effect helpers, and the global animation-disable behavior. Expensive textures, glows, and motion must degrade consistently at medium/low quality.
- Keep animation purposeful and brief, and ensure essential progress feedback still works with decorative animations disabled.
- Use icons from the existing icon/assets pipeline and `assets::img_asset`; do not inline a second icon system or hard-code deployment-sensitive asset URLs.
- Prefer consistent density over oversized mobile-app controls. Dense ARPG information must remain scannable, but interactive targets still need to be usable with touch and keyboard.
- Treat a new shared visual pattern as a component, not a copied block of Tailwind classes. If the same nontrivial pattern appears a second time, extract it.

- Keep components focused; move reusable calculations out of view macros.
- Maintain keyboard access, labels, focus behavior, readable contrast, and non-color-only state cues.
- Make layouts usable at narrow browser widths as well as desktop sizes.
- Avoid optimistic rendering for irreversible inventory, currency, market, or progression operations unless rollback is explicit.
- Reuse tooltip and number-formatting components so dense ARPG statistics stay consistent.
- Do not add large dependencies for behavior that existing Rust, Leptos, or CSS utilities already provide.

## Backend conventions

Keep HTTP routing/transport in `rest` or `websocket`, persistence in `db`, orchestration in `game`, and individual rules in focused `game/systems` modules. Reuse `AppState`, shared error handling, and existing controller/updater patterns.

- Do not hold locks across `.await` points.
- Return contextual errors at service boundaries; convert them to safe client-facing messages without leaking internals.
- Use parameterized SQL only.
- Preserve idempotency where retries are plausible, especially for rewards, purchases, market operations, and session finalization.
- Add tracing that identifies the operation and safe entity IDs, not secrets or entire payloads.

## Working method

Before editing, read the target file, its module entry point, and a nearby analogous implementation. Search references to any type, resource key, message variant, or database column you change.

For an explicitly requested gameplay feature, normally consider this full slice:

1. Domain types and pure calculations in `shared`.
2. JSON definitions and manifests in `data`, if content-driven.
3. Loading, validation, and authoritative behavior in `backend`.
4. Client/server message changes in `shared`, if state crosses the socket.
5. Rendering and player input in `frontend`.
6. Persistence and paired migrations, if state survives a session.
7. Focused tests for formulas, boundary conditions, validation, and serialization.

Keep patches focused. Do not reformat unrelated files, rename broad APIs incidentally, edit generated build output, or replace user-authored changes. Prefer extending established patterns over introducing a parallel abstraction.

## Validation

Run the smallest useful checks during development, then broaden them in proportion to the change.

```powershell
# Format the workspace
cargo fmt --all -- --check

# Fast native checks
cargo check --workspace
cargo test --workspace

# Lint native/default-feature code
cargo clippy --workspace --all-targets -- -D warnings

# Check the browser target after frontend or shared changes
rustup target add wasm32-unknown-unknown
cargo check -p frontend --target wasm32-unknown-unknown

# Build the web app, including its CSS pipeline
Set-Location frontend
trunk build
```

Backend CI validates PostgreSQL explicitly and requires a migrated test database:

```powershell
$env:DATABASE_URL = $env:TEST_DATABASE_URL
cargo sqlx migrate run --source migrations/postgres
cargo sqlx prepare --workspace -- --bin backend --features postgres --no-default-features
cargo clippy --no-default-features --features postgres -- -D warnings
cargo test --no-default-features --features postgres
```

Do not claim a PostgreSQL or Trunk check passed if its required service/tool was unavailable. State exactly which commands ran and any environmental limitation.

Add or update tests near the behavior they protect. Prioritize tests for:

- modifier stacking and local/global scope;
- damage, mitigation, cooldown, resource, and reward boundaries;
- malformed or stale client input;
- loot eligibility and inventory capacity;
- save/message serialization compatibility;
- cross-referenced data loading;
- migration behavior for both database engines.

For UI changes, also inspect the affected flow in a browser when available. Compare it beside an adjacent existing screen and verify that typography, surfaces, controls, spacing, colors, and interaction states feel like one system. Test loading, empty, error, disabled, selected, hover/focus, and narrow-screen states rather than only the happy path. Check both high and low graphics quality and, for animated UI, the animations-disabled setting.

## Completion checklist

Before handing off a change:

- The implementation lives in the correct crate/layer and preserves server authority.
- No gameplay balance or content changed without explicit user direction; unrelated work preserves existing behavior and values.
- Content IDs and manifest entries resolve, and new assets use valid repository-relative paths.
- Protocol and persisted-state changes are compatible or deliberately migrated.
- SQLite and PostgreSQL schemas remain aligned.
- Formatting and relevant checks/tests have run.
- Frontend work reuses the existing component hierarchy and matches neighboring screens across responsive and graphics-quality modes.
- Player-facing mechanics and formulas are understandable in the UI.
- New creative content is original and does not imitate protected Path of Exile material too closely.
- The final report names changed areas, validation performed, and any remaining risk or follow-up.
