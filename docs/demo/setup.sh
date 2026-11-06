# Sourced by docs/demo/demo.tape: a small Claude Code repo in a temp dir.
D=$(mktemp -d)/billing-api && mkdir -p "$D" && cd "$D"
git init -q -b main && git config user.name demo && git config user.email demo@example.com
mkdir -p .claude/commands .claude/agents
printf '# Billing API\n\nTypeScript service, Postgres.\n\n## Commands\n\n- Test: `pnpm test`\n' > CLAUDE.md
printf -- '---\ndescription: Review the current diff\n---\nReview the diff for $ARGUMENTS.\n' > .claude/commands/review.md
printf -- '---\nname: tester\ndescription: Writes tests\n---\nYou write tests first.\n' > .claude/agents/tester.md
printf '{"mcpServers":{"postgres":{"command":"npx","args":["-y","@modelcontextprotocol/server-postgres"],"env":{"DATABASE_URL":"${DATABASE_URL}"}}}}\n' > .mcp.json
git add -A && git commit -qm init
export PS1='\[\e[1;36m\]billing-api\[\e[0m\] $ '
