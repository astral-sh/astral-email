Rewrite only the newest release section in `CHANGELOG.md`: from its first `## `
heading to the next release heading or end of file. Read the changelog and local
Git history; do not edit files or use the network. Match several preceding
releases' section names, ordering, tone, and Markdown style when available.
Inspect local changes when generated titles do not explain an entry.

Apply these rules:

- Preserve the release version and date.
- Preserve existing reference formatting and every retained entry's pull request
  number and exact URL. Never rewrite URLs.
- Keep each paragraph and list item, including its links, on one physical line.
  Shorten long prose or split independent changes instead of hard-wrapping.
- Drop changes with no user-facing effect, such as CI, test runners, repository
  reorganization, and agent or developer infrastructure. Keep uncertain entries.
- Never move entries between `Enhancements` and `Bug fixes` unless splitting
  distinct changes. Never move a bug fix to `Performance`.
- Keep one independently useful change per bullet. Combine duplicate changes,
  preserving all references. Split distinct changes, repeating the exact pull
  request number and URL in each entry; use different sections only when each
  change unambiguously belongs there. Do not split implementation steps.
- Move `Enhancements` or `Other changes` entries to `Performance` only when
  performance is their primary intent. Move other entries out of `Other changes`
  only when an established section clearly fits. Otherwise retain user-relevant
  maintenance there, including MSRV, toolchain, and downstream API compatibility.
- Rewrite generated wording for clarity and precision. Expand internal shorthand
  and add context supported by local changes, preserving meaning without inventing
  or broadening claims. Avoid purely stylistic synonym changes.
- Lead with what users can do or what behavior is fixed. Prefer one short sentence
  per bullet and brief introductions. Omit incidental implementation details and
  exhaustive flag or edge-case lists; retain qualifiers such as opt-in behavior
  and affected platforms.
- Order each section by significance and remove empty sections.

Return only the complete replacement section, beginning with its single `## `
heading. Use `### ` for subsections. Omit older releases, code fences, and commentary.
