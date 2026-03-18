# AGENTS.md

# Rust / verifyOS-cli

Guidelines for developing the `verifyOS-cli` App Store rejection risk scanner.

## Core Principles

- **"Shift Left" Validation**: Catch App Store Connect rejections as early as possible on the developer's local machine or CI runner.
- **Local Analysis**: Do not transmit app bundles to external servers. All analysis must be performed locally using Rust.
- **Zero Placeholders**: Implement actual business logic for App Store rules. If a rule is requested, implement the core engine logic or use established patterns.

## Code Style & Conventions

- **Inlined Variables**: Always inline variables into `format!` strings (`format!("{var}")`) where possible.
- **Clippy Compliance**: 
    - Always collapse `if` statements when possible.
    - Use method references over closures when possible (`.map(|x| x.foo())` -> `.map(X::foo)`).
- **Match Statements**: Ensure `match` statements are exhaustive. Avoid wildcard arms (`_`) if the set of variants is known and stable.
- **Modules**:
    - Target module files under 500 lines of code.
    - Follow the established directory structure:
        - `core/`: Orchestration and top-level pipeline logic (Engine).
        - `parsers/`: File format handles (`zip`, `plist`, `macho`).
        - `rules/`: Trait-based validation logic (`AppStoreRule`).

## Rule Engine & Traits

- **Trait Usage**: All validation rules must implement the `AppStoreRule` trait found in `src/rules/core.rs`.
- **Context Awareness**: Use the `ArtifactContext` to access `Info.plist` data and the app bundle path during evaluation.
- **Error Handling**: 
    - Use `miette` for user-facing diagnostics. 
    - Use `thiserror` for internal rule-specific errors (`EntitlementsError`, `PlistError`).
    - Use absolute `verifyos_cli::` paths for cross-module imports.

## Data Processing Rules

- **IPAs/App Bundles**: Treat `.ipa` files as ZIP archives. Extract to temporary directories and clean up after analysis.
- **Info.plist**: Use the `plist` crate for robust parsing. Map keys to structured `InfoPlist` structs in `src/parsers/plist_reader.rs`.
- **Mach-O**: Use `goblin` or `apple-codesign` for deep binary inspection (entitlements, architectures).

## Testing

- **Integration Tests**: Place major test flows in `tests/cli_test.rs` and `tests/rules_test.rs`.
- **Fixtures**: Use the `.ipa` and `.app` fixtures in `examples/` and `tests/fixtures/`.
- **Path Resolution**: Use `CARGO_MANIFEST_DIR` base paths to locate test fixtures reliably across different environments.

## Tooling

- **Cargo**: Standard commands: `cargo fmt`, `cargo clippy`, `cargo test`.
- **Release**: Follow conventional commits (`feat:`, `fix:`, `chore:`, `docs:`) to facilitate the `release-plz` workflow.

---

## Memory System: cass-memory

The Cass Memory System (cm) is a tool for giving agents an effective memory based on the ability to quickly search across previous coding agent sessions across an array of different coding agent tools (e.g., Claude Code, Codex, Gemini-CLI, Cursor, etc.) and projects (and even across multiple machines, optionally) and then reflect on what they find and learn in new sessions to draw out useful lessons and takeaways; these lessons are then stored and can be queried and retrieved later, much like how human memory works.

The `cm onboard` command guides you through analyzing historical sessions and extracting valuable rules.

### Quick Start

```bash
# 1. Check status and see recommendations
cm onboard status

# 2. Get sessions to analyze (filtered by gaps in your playbook)
cm onboard sample --fill-gaps

# 3. Read a session with rich context
cm onboard read /path/to/session.jsonl --template

# 4. Add extracted rules (one at a time or batch)
cm playbook add "Your rule content" --category "debugging"
# Or batch add:
cm playbook add --file rules.json

# 5. Mark session as processed
cm onboard mark-done /path/to/session.jsonl
```

Before starting complex tasks, retrieve relevant context:

```bash
cm context "<task description>" --json
```

This returns:
- **relevantBullets**: Rules that may help with your task
- **antiPatterns**: Pitfalls to avoid
- **historySnippets**: Past sessions that solved similar problems
- **suggestedCassQueries**: Searches for deeper investigation

### Protocol

1. **START**: Run `cm context "<task>" --json` before non-trivial work
2. **WORK**: Reference rule IDs when following them (e.g., "Following b-8f3a2c...")
3. **FEEDBACK**: Leave inline comments when rules help/hurt:
   - `// [cass: helpful b-xyz] - reason`
   - `// [cass: harmful b-xyz] - reason`
4. **END**: Just finish your work. Learning happens automatically.

### Key Flags

| Flag | Purpose |
|------|---------|
| `--json` | Machine-readable JSON output (required!) |
| `--limit N` | Cap number of rules returned |
| `--no-history` | Skip historical snippets for faster response |

stdout = data only, stderr = diagnostics. Exit 0 = success.

## Repo Rules

- Keep commits small. Prefer one behavioral change per commit.
- Split work into separate commits when it improves reviewability:
  - `feat:` for behavior
  - `test:` for meaningful test additions or coverage changes
  - `docs:` for README, architecture, examples, or agent-playbook updates
  - `chore(ci):` for workflow or automation changes
- Do not bundle unrelated code, tests, docs, and CI changes into one large commit unless they must ship together for correctness.
- Before every push, run:
  - `cargo fmt --all -- --check`
  - `cargo test`
  - `cargo clippy --all-targets --all-features -- -D warnings`
- Keep conventional commit messages concise and scope-specific.

---

## Frontend (Taste Skill)

All frontend development MUST adhere to the `taste-skill` design philosophy for a premium, high-performance UI.

### Core Configuration
- **DESIGN_VARIANCE**: 8 (Modern, asymmetric, overlapping elements).
- **MOTION_INTENSITY**: 6 (Fluid transitions, staggered reveals, tactile feedback).
- **VISUAL_DENSITY**: 4 (Spacious, "Art Gallery" feel, luxury spacing).

### Design Rules
- **Anti-Slop**: Banned "AI Purple/Blue" glows and generic 3-column card layouts.
- **Typography**: Strictly use `Geist`, `Outfit`, or `Satoshi` for Sans-Serif. Serif fonts are BANNED for software UIs.
- **Icons**: Exclusively use `@phosphor-icons/react` or `@radix-ui/react-icons`. **Emojis are BANNED**.
- **Materiality**: Use "Liquid Glass" refraction (1px inner borders + tinted shadows). Omit physical cards/boxes where spacing can define grouping.
- **Viewport Stability**: ALWAYS use `min-h-[100dvh]` for full-height sections to prevent mobile layout jumping.

### Animation & Performance
- **Framer Motion**: Use for UI/Bento interactions. Use `layout` and `layoutId` for smooth state transitions.
- **Spring Physics**: Use `type: "spring", stiffness: 100, damping: 20` for a premium, weighty feel.
- **Hardware Acceleration**: Animate ONLY `transform` and `opacity`. Use `will-change` sparingly.
- **Isolation**:CPU-heavy perpetual animations MUST be isolated in their own microscopic Client Components.
