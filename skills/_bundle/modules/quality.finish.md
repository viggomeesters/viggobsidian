# quality.finish

Resolve relative links from this module file’s directory, after resolving profile symlinks.

Use for rechecking just-delivered work or explicitly requested bounded repairs. This is not a default audit phase. Select only the matching procedure; do not load the whole family.

Read `../contracts/routing.md` when deciding between effects or owners.

Choose recheck for the attack → repair → verify loop on just-delivered work; hardening-pass for an explicitly requested bounded repair pass. improve is its alias. Read-only wording disables repairs.

Explicit recheck/finish preserves authorized in-scope repair. An ordinary audit does not select this executing mode.

- `recheck`: [recheck](../../recheck/SKILL.md)

- `hardening-pass`: [hardening-pass](../../hardening-pass/PROCEDURE.md)

- `improve`: [improve](../../improve/PROCEDURE.md)
