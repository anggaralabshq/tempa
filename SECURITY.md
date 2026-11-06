# Security

Please report vulnerabilities privately through [GitHub security advisories](https://github.com/anggaralabshq/tempa/security/advisories/new), not in a public issue.

Include the tempa version, your platform, and steps to reproduce. You can expect a first answer within a week. Fixes go into the next release; only the latest release is supported.

tempa writes files inside your repository and registers an MCP server (`tempa mcp`) that agents start locally. Issues in either area, such as writing outside the repository, overwriting a user's file without `--force`, or an MCP tool acting outside the contract, are in scope.
