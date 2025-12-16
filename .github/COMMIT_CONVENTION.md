# Git Commit Message Convention

> This convention is adapted from [Angular's commit convention](https://github.com/conventional-changelog/conventional-changelog/tree/master/packages/conventional-changelog-angular)
> and inspired by [discord.js' commit convention](https://github.com/discordjs/discord.js/blob/main/.github/COMMIT_CONVENTION.md)
>
> ForgeKit follows the same overall structure with project-specific scopes and
> rules below.

## TL;DR

Messages must match the following regex:

```js
/^(revert: )?(feat|fix|docs|style|refactor|perf|test|build|ci|chore|types)(\([^)]+\))?!?: .{1,72}$/;
```

## Examples

Appears under "Features" header, `cli` scope:

```text
feat(cli): add --force flag to overwrite existing files (#22)
```

Appears under "Bug Fixes" header, `core` scope, closing an issue:

```text
fix(core): prevent page range parsing overflow

Closes #12
```

Appears under "Performance Improvements" header, and under "Breaking Changes":

```text
perf(core)!: improve PDF merging by removing 'bar' option

BREAKING CHANGE: The 'bar' option has been removed.
```

Reverts do not appear in the changelog if they are under the same release
as the reverted commit. If not, the revert commit appears under the
"Reverts" header.

```text
revert: feat(cli): add --force flag to overwrite existing files

This reverts commit 667ecc1654a317a13331b17617d973392f415f02.
```

## Full Message Format

A commit message consists of a **header**, **body**, and **footer**. The
header has a **type**, **scope**, and **subject**:

```text
<type>(<scope>)!: <subject>

<BLANK LINE>

<body>

<BLANK LINE>

<footer>
```

The **header** is mandatory and the **scope** of the header is optional.
If the commit contains **Breaking Changes**, a `!` can be added before the
`:` as an indicator.

## Revert

If the commit reverts a previous commit, it should begin with `revert:`,
followed by the header of the reverted commit. In the body, it should say:

```text
This reverts commit <hash>.
```

## Type

The `type` must be one of:

- `feat`: new feature
- `fix`: bug fix
- `docs`: documentation changes
- `style`: formatting only (no code behavior change)
- `refactor`: refactor (no user-visible behavior change)
- `perf`: performance improvement
- `test`: tests only
- `build`: build system / dependencies
- `ci`: CI/workflow changes
- `chore`: maintenance tasks
- `types`: type-only changes (if applicable)

Notes:

- If the prefix is `feat`, `fix`, or `perf`, it will appear in the changelog.
- If there is any [BREAKING CHANGE](#breaking-changes), the commit will always
  appear in the changelog.

## Scope

The scope could be anything specifying the place of the change.

Recommended scopes for ForgeKit:

- `cli`: CLI commands and interface
- `core`: Core library functionality
- `tools`: Tool integrations (qpdf, pdfcpu, tesseract, etc.)
- `pdf`: PDF-specific operations
- `image`: Image operations (for future features)
- `media`: Media operations (for future features)
- `docs`: Documentation
- `build`: Build system
- `ci`: CI/workflow changes
- `packaging`: Package manager configurations (Homebrew, winget, debian, rpm)

Examples:

```text
fix(core): handle errors in job executor
feat(cli): support page range exclusions
fix(tools): improve qpdf error detection
feat(packaging): add Homebrew formula
```

## Subject

The subject contains a succinct description of the change:

- use the imperative, present tense: "change" not "changed" nor "changes"
- don't capitalize the first letter (unless proper noun/acronym)
- no dot (`.`) at the end
- keep the header at 72 characters max (per regex)

## Body

Just as in the **subject**, use the imperative, present tense: "change"
not "changed" nor "changes".

The body should include the motivation for the change and contrast this
with previous behavior.

## Footer

The footer should contain any information about **Breaking Changes** and
is also the place to reference GitHub issues that this commit **Closes**
(or otherwise references).

### Issue references

Use GitHub keywords so issues close automatically when appropriate:

```text
Closes #12
Fixes #17
Refs #18
```

### Breaking Changes

Breaking changes should start with the words `BREAKING CHANGE:` followed
by an explanation:

```text
feat(cli)!: change PDF merge output flag from --out to --output

BREAKING CHANGE: The --out flag has been renamed to --output for consistency.
```

