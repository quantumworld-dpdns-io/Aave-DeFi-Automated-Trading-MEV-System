AVAILABLE_SKILLS=
  agents-sdk, cloudflare, cloudflare-email-service, customize-opencode, durable-objects, sandbox-sdk, turnstile-spin, web-perf, workers-best-practices, wrangler

def build_tool_calls():
    # Build comprehensive tool call list for entire implementation plan
    calls = []
    
    # ==========================================
    # 1. Write contribution guideline
    # ==========================================
    calls.append({
        "tool": "write",
        "filePath": "CONTRIBUTING.md",
        "content": "# Contribution Guidelines

## Project Structure
This project uses a monorepo structure with:

```
contracts/          # Solidity contracts
  src/             # Contract source code
  test/            # Foundry tests
  script/          # Deployment scripts
  foundry.toml     # Foundry configuration

services/executor-rust/    # Mempool monitoring and transaction execution
  src/
  Dockerfile
  Cargo.toml

services/gateway-go/       # API Gateway and Watchdog
  cmd/
  internal/
  Dockerfile
  go.mod

services/quant-python/     # Quant Analytics and Risk Models
  src/
  Dockerfile
  requirements.txt

.choreo/                   # Choreo.dev deployment config
  endpoints.yaml
  component.yaml

docker-compose.yml         # Local development setup
Dockerfile*               # Docker images for each service
.GHA/                     # GitHub Actions workflows
.gromanport/               # OCR configuration
```

## Development Workflow

### 1. Make a Setup Commit

Before starting any feature development, create a baseline commit that includes all project tooling:

```bash
# 1. Initialize git repository
mkdir Aave-DeFi-Automated-Trading-MEV-System
cd Aave-DeFi-Automated-Trading-MEV-System
git init
git add .
git commit -m "Initial setup with project structure and tooling"
```

### 2. Environment Setup

This project uses Docker for development. Once you have Docker installed, you can start with the local development environment:

```bash
# Build and start all containers (contracts, gateway, executor, quant)
docker-compose up --build -d

# Follow logs
docker-compose logs -f
```

The stack includes:
- **Anvil**: Foundry fork of Base mainnet
- **Executor** (Rust): Mempool monitoring service
- **Gateway** (Go): API Gateway and Watchdog service
- **Quant** (Python): Quant analytics service

### 3. Running Tests

All tests are run locally for now (cloud rollout in Phase 4+):

```bash
# 1. Contract tests (Foundry)
cd contracts
forge test

# 2. Rust tests
cd ../services/executor-rust
docker run --rm mev-system/executor-rust cargo test

# 3. Go tests
cd ../services/gateway-go
go test ./...

# 4. Python tests
cd ../services/quant-python
poetry run pytest tests/ -x
```

### 4. Running Integration Tests

```bash
# Requires Anvil fork (automatic with docker-compose)
cd ../
docker run --rm --network=network_default mev-system/executor-rust cargo test integration
```

### 5. Testing a Change

Each PR should pass all CI checks, including:

```bash
# Run all tests together
$(MAKE) test-unit

# Or run integration tests
$(MAKE) test-integration
```

### 6. Local Testing Preview

For quick local testing without Docker:

**Solidity:**
```bash
cd contracts
npx hardhat test
```

**Rust:**
```bash
cd services/executor-rust
docker run -it --network=network_default mev-system/executor-rust cargo run --example mempool
```

**Go:**
```bash
cd services/gateway-go
go run ./cmd/server
```

**Python:**
```bash
cd services/quant-python
poetry run python -m quant.main
```

## Testing Protocols

### Contract Testing

Automated contract testing with Foundry:

**1. Unit Tests**
- Hard-coded scenarios
- Edge case coverage
- Property-based testing with fuzzing

**2. Fork Tests**
- Test against Base mainnet fork (Alchemy/QuickNode)
- Real interactions with live protocol state
- MeV opportunities

### Integration Testing

RobotFramework ensures end-to-end workflow:

```robot
*** Test Cases ***
END-TO-END TRADING WORKFLOW
    [Documentation]    Quant creates opportunity -> Gateway validates -> Executor simulates -> Paper trade
    Start Full Stack
    Quant Tests Strategy
    Gateway Risk Check
    Executor Simulate
    Verify Paper Trade
    Stop Full Stack
```

### Coverage Requirements

Test coverage targets (coverage calculated before mainnet deployment):

| Phase | Coverage Goal |
|-------|---------------|
| Phase 1 Contract Tests | 90%+ coverage on critical functions |
| Phase 2 Unit Tests | 80%+ coverage on smart contracts |
| Phase 3 Integration Tests | 100% end-to-end workflow |
| Phase 4 Fork Tests | 95%+ coverage on prod simulation |

### Security Testing

**Pre-Contract Audit Requirements:**
- [x] Static analysis with Slither and Mythril
- [x] Fuzzing with Echidna for invariants
- [x] Property testing with Foundry's fuzzing
- [ ] Formal verification on core risk model (Phase 5+)

### Test Quality Criteria

Each test must meet these criteria:

1. **Deterministic**: Same inputs always produce same outputs
2. **Isolated**: One test failure cannot affect others
3. **Complete**: Tests cover at least 90% of contract code
4. **Realistic**: Simulates real blockchain interactions
5. **Fast**: Tests run within reasonable time limits
6. **Maintainable**: Clear assertions and readable structure

### Pull Request Review Checklist

Before merging any PR, all team members should:

**Code Review:**
- [ ] Tests pass locally and in CI
- [ ] Test coverage not reduced
- [ ] Documentation updated
- [ ] Comments added for complex logic
- [ ] No hardcoded values or passwords
- [ ] Follows project style conventions
- [ ] No performance issues (gas optimization)

**Security Review:**
- [ ] Manual security review completed
- [ ] Static analysis approved
- [ ] Fuzzing results reviewed
- [ ] No access control issues

**Collaboration Review:**
- [ ] Reviewer approved in PR description
- [ ] After PR merge, all team members can run and test the code
- [ ] No single developer lock on any part of the codebase

## Detailed Rules

### One Responsibility Per File
Each file should have one clear responsibility:

- Imports: No more than 20 imports per file; sort by source (standards, internal), sort import groups by source.
- Style: Follow the _explicit_ style: explicitly construct types; avoid unwrapping or casting;
- API: Align APIs with typical usage patterns; graceful handling of errors;
- Documentation: Each function at least has 2-3 line doc comment explaining its purpose.

### Code Quality

**Naming**

1. Function and method names should follow `snake_case`.
2. Type alias names should start with uppercase letter.
3. Variable names should be `snake_case`.
4. Constant name should be `UPPER_CASE_WITH_UNDERSCORES`.

**Formatting**

1. Always use double quotes for strings.
2. Indent with 4 spaces.
3. Add 2 blank lines between logical sections.

**Error Handling**

1. Return errors rather than panic; include meaningful messages.
2. Use appropriate error types and avoid `expect()`.
3. Log errors with necessary context.

**Testing**

1. Test both success and failure cases.
2. Use property-based testing for mathematical operations.
3. Edge cases are important.
4. Mock dependencies for unit tests.

### PR Practices

1. **Pre-commit check**: Avoid uncommitted changes when creating PR.
2. **Tag PR name**: Preferred format `[feature-name | contract-fix]`, with optional context.
3. **One PR for one feature and one issue**: A single PR should accomplish one feature or fix one issue if possible.
4. **Merge to main**: All PRs should merge to the `main` branch, no feature branches to merge (except for merge requests).
5. **Squash pull request**: PRs from personal branches should be squashed before merging.
6. **Pull request review**: Ensure formal reviews for at least one hour.

### Branch Management

1. **Branch naming**: All development branches start with feature name: `feature/<module>/sub-module/...<desc>`.
2. **Branch merge**: Always use PRs to merge any branch to branch `main`.
3. **Branch protection**: PR need at least one person (developer or maintainer) to approve and discuss.

### Version Control

1. **Commit message**: `git log --pretty=fuller -1` to review.
2. **Squash commits**: Must squash commits on your topic branch before merge.
3. **Rebasing**: Always rebase your topic branch from `main`.
4. **Review commits**: Use clean, concise, descriptive commit messages (less than 50 characters).
5. **Write messages carefully**:
   - Use imperative, present tense: 'Do not merge code that is complicated.'
   - Use issue reference: 'Fix #123'.
   - No commit message: 'Fix #123 Problem'.
   - Include suggested-by: 'Suggested-by: OpenHands <openhands@all-hands.dev>'.

### Developer Contribution

1. **Code when you want to stop**: Code when you want to continue.
2. **Commit smoke tests**: Smoke tests should be committed before or with code for the first time.
3. **Test first**: Git commit should be a test that fails, then code to make it pass, then commit code.
4. **Verify propagation**: Never trust your code won't break later.
5. **Code review after each commit**: Explicit `git review` command by another team member.
6. **Uncommitted broken**: Never commit broken code.
7. **Branch boundary**: Test before migrating to other branches.

### Repository Structure Best Practices

1. **Single responsibility**: Each directory should contain related files; if not, move to better location.
2. **Consistent structure**: Your directory structure should follow similar patterns across the project.
3. **No random files**: No files which might drift in the repo with no purpose.
4. **Not reusing existing test repo**: Don't reuse existing public test repo for your project. Every project has its own test repository.
5. **Test configuration**: Use a robust, well set up project template with dedicated test configuration files. Better to keep tests outside the main source tree.

## Command Examples

### Starting the Environment

```bash
# Build and start all containers
docker-compose up --build

# See logs
docker-compose logs -f

# Stop everything
docker-compose down
```

### Running Tests

```bash
# Contract tests
cd contracts
forge test

# Go tests
cd services/gateway-go
go test ./...

# Python tests
cd services/quant-python
poetry run pytest tests/ -x

# Integration tests
$(MAKE) test-integration
```

### Building Docker Images

```bash
$(MAKE) build
```

### Running System Status

```bash
$(MAKE) status
```

### Running Health Checks

```bash
# Check service health
$(MAKE) healthcheck
```

### Cleaning Up

```bash
$(MAKE) delete
```

## Commit Guidelines

- Updates to documentation files should include update commit messages with updated sections descriptions
- Updates to gitignore should include update commit messages with modified sections descriptions
- When updating any kind of files, updates should be included in commits with proper commit messages

## Discord Integration

All PRs should be announced in Discord, and team members should arrive for discussions if needed.

## Next Steps

1. Fork this repository
2. Make changes and create a PR for each completed task
3. Test changes thoroughly before merging
4. Follow the PR review checklist
