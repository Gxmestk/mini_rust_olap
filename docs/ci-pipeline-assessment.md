# CI/CD Pipeline Assessment

## 📋 Assessment Overview

This assessment tests your understanding of the CI/CD pipeline created for Mini Rust OLAP. The pipeline uses local git hooks to enforce code quality standards and ensure only tested code reaches remote repositories.

### Assessment Structure

| Part | Topic | Questions | Difficulty |
|-------|--------|------------|
| 1 | Git Hooks Fundamentals | 8 | Beginner |
| 2 | Pre-commit Hook Details | 10 | Beginner/Intermediate |
| 3 | Pre-push Hook Details | 10 | Intermediate |
| 4 | Troubleshooting & Best Practices | 7 | Intermediate/Advanced |
| **Total** | | **35** | |

### Scoring Guide

- **30-35**: Excellent understanding - ready for advanced CI/CD concepts
- **25-29**: Good understanding - review hook details
- **20-24**: Fair understanding - review all topics
- **Below 20**: Needs review - revisit CI pipeline materials

---

## Part 1: Git Hooks Fundamentals

### Q1. What is the primary purpose of git hooks?

A. To automatically commit changes
B. To run automated checks or actions at specific git events
C. To manage remote repository connections
D. To format git commit messages

### Q2. When does the pre-commit hook run?

A. After pushing to remote repository
B. Before creating a new commit
C. After pulling changes from remote
D. When running git status

### Q3. When does the pre-push hook run?

A. Before creating a new commit
B. Before pushing to a remote repository
C. After receiving changes from remote
D. After merging pull requests

### Q4. Which git command can be used to bypass git hooks temporarily?

A. git commit --skip
B. git commit --no-verify
C. git push --force
D. git commit --amend

### Q5. Where are git hooks stored in a git repository?

A. In the .gitignore file
B. In the .git/hooks/ directory
C. In the .github/workflows/ directory
D. In the src/ directory

### Q6. What happens if a git hook exits with a non-zero status code?

A. The hook is ignored and the git operation continues
B. The git operation is aborted and the commit/push fails
C. A warning is shown but the operation continues
D. The hook is disabled for future operations

### Q7. Which command is used to make a hook file executable?

A. git make-executable <hook-file>
B. git hook <hook-file> --mode=755
C. git config core.hooksPath <hook-file>
D. chmod +x <hook-file>

### Q8. What is the difference between local git hooks and GitHub Actions?

A. There is no difference - they are the same thing
B. Local hooks run on your machine, GitHub Actions runs on GitHub servers
C. Local hooks are for testing, GitHub Actions are for production
D. Local hooks use bash scripts, GitHub Actions use YAML configuration

---

## Part 2: Pre-commit Hook Details

### Q9. Which cargo command checks if code follows Rust's standard formatting?

A. cargo check
B. cargo fmt --all -- --check
C. cargo clippy
D. cargo test

### Q10. What is the purpose of cargo clippy?

A. To compile the code with optimizations
B. To check code for common mistakes and suggest improvements
C. To run unit tests
D. To generate documentation

### Q11. Which command verifies code compiles without producing binaries?

A. cargo build
B. cargo check --all-targets
C. cargo test
D. cargo run

### Q12. What does the --all-targets flag do in cargo check?

A. Runs checks for all dependencies in the workspace
B. Compiles all targets including benchmarks and examples
C. Compiles all targets except for documentation
D. Compiles all targets in release mode

### Q13. Why does the pre-commit hook run integration tests with --test flag?

A. Because unit tests are not sufficient
B. To separate unit tests from integration tests
C. To test multiple test binaries simultaneously
D. Because --test is faster than running cargo test without flags

### Q14. What happens if cargo fmt -- --check finds unformatted code?

A. The hook automatically formats the code and continues
B. The hook shows a warning but allows the commit
C. The hook runs cargo fmt --all to fix formatting and shows changed files
D. The hook aborts the commit and shows which files need formatting

### Q15. Which cargo command checks if documentation compiles without warnings?

A. cargo doc
B. cargo doc --no-deps
C. cargo doc --document-private-items
D. cargo check --doc

### Q16. Why does the pre-commit hook run cargo check before cargo test?

A. To save time by not testing uncompiled code
B. To generate documentation before testing
C. To format the code before testing
D. Because cargo test implicitly checks compilation

### Q17. What is the approximate time for the pre-commit hook to complete?

A. Less than 5 seconds
B. 30-60 seconds
C. 2-5 minutes
D. 5-10 minutes

### Q18. Which of the following is NOT checked by the pre-commit hook?

A. Code formatting
B. Clippy linting
C. Compilation errors
D. Release mode tests

---

## Part 3: Pre-push Hook Details

### Q19. Why does the pre-push hook run cargo check --release?

A. To generate documentation in release mode
B. To ensure release optimizations don't introduce bugs
C. To run tests faster without optimizations
D. To create a release binary

### Q20. What is the main difference between pre-commit and pre-push hooks in terms of checks performed?

A. Pre-commit checks are faster, pre-push checks are slower
B. Pre-commit checks formatting, pre-push checks generated files
C. Pre-commit runs in debug mode, pre-push runs in release mode
D. Pre-commit checks basic functionality, pre-push checks edge cases

### Q21. Which git command does the pre-push hook use to check for uncommitted changes in target/ directory?

A. git diff --name-only
B. git status
C. git ls-files target/
D. git diff --cached

### Q22. Why might release mode tests fail when debug mode tests pass?

A. Debug mode has more checks enabled
B. Release optimizations can change behavior subtly
C. Release mode disables assertions that might have bugs
D. Release mode uses different memory layouts that expose issues

### Q23. What tool does the pre-push hook use to measure code coverage?

A. cargo bench
B. cargo test -- --nocapture
C. cargo-tarpaulin
D. cargo cov

### Q24. Which check in the pre-push hook is considered "informational" and not blocking?

A. Code coverage check
B. TODO/FIXME comment check
C. README examples check
D. Documentation completeness check

### Q25. What does the pre-push hook check regarding Cargo.lock?

A. If Cargo.lock exists and is up to date
B. If Cargo.lock follows the naming conventions
C. If Cargo.lock is properly formatted
D. If Cargo.lock is committed with Cargo.toml changes

### Q26. Why does the pre-push hook check for TODO/FIXME comments?

A. To count them for metrics
B. To prevent commits with incomplete work
C. To warn developers about pending work
D. It's a warning, not a blocking check

### Q27. What does the README examples check verify?

A. That all code examples in README compile successfully
B. That README follows markdown formatting rules
C. That README contains at least 10 code examples
D. That README examples are present and syntactically valid

### Q28. Which additional validation does the pre-push hook perform compared to pre-commit?

A. Checks for uncommitted changes in generated files
B. Verifies code coverage (optional)
C. Validates README examples
D. Checks documentation completeness
E. All of the above

---

## Part 4: Troubleshooting & Best Practices

### Q29. If a git hook is not running, what is the first thing to check?

A. Verify the hook file exists in .git/hooks/ directory
B. Check if the hook file is executable
C. Verify core.hooksPath is set correctly
D. All of the above

### Q30. What is a common reason for "permission denied" errors when running git hooks?

A. Hook file is not executable
B. Git repository has incorrect permissions
C. User doesn't have write access to .git/hooks/ directory
D. Hook script tries to access system directories without permission

### Q31. How can you temporarily bypass a failing hook to commit urgent code?

A. git commit --amend --no-verify
B. git commit --skip
C. git push --no-verify
D. git commit -m "message" --allow-empty

### Q32. What is the recommended practice for fixing hook failures?

A. Bypass the hook and fix later
B. Fix the issue, stage the fix, and commit again
C. Delete the hook file
D. Run hooks manually and continue regardless of result

### Q33. Why might you want to make pre-commit hooks run faster?

A. To reduce friction during development
B. To encourage smaller, more frequent commits
C. To reduce waiting time for large projects
D. Because pre-commit hooks run on every commit attempt

### Q34. What is the benefit of running tests locally with hooks before pushing?

A. Faster iteration - get feedback in seconds, not minutes
B. Prevents broken code from reaching remote repository
C. Reduces load on CI/CD servers
D. Allows for more detailed debugging on your own machine

### Q35. Which git command shows the status of installed hooks?

A. git hook list
B. git hooks --list
C. git config --get core.hooksPath
D. git status

---

## 📊 Answer Key

### Part 1: Git Hooks Fundamentals

| Question | Correct Answer | Explanation |
|----------|----------------|-------------|
| Q1 | **B** | Git hooks run automated checks or actions at specific git events (like pre-commit, pre-push) to enforce quality standards |
| Q2 | **B** | The pre-commit hook runs before creating a new commit, making it the first line of defense |
| Q3 | **B** | The pre-push hook runs before pushing to a remote repository, acting as a final gatekeeper |
| Q4 | **B** | git commit --no-verify bypasses all hooks for that specific commit only |
| Q5 | **B** | Git hooks are stored in the .git/hooks/ directory (created by setup script) |
| Q6 | **B** | If a hook exits with non-zero, the git operation is aborted, preventing low-quality commits |
| Q7 | **D** | chmod +x <hook-file> sets the executable permission on the hook file |
| Q8 | **B** | Local hooks run on your machine, while GitHub Actions run on GitHub's servers |

### Part 2: Pre-commit Hook Details

| Question | Correct Answer | Explanation |
|----------|----------------|-------------|
| Q9 | **B** | cargo fmt --all -- --check verifies code follows Rust's standard formatting without modifying it |
| Q10 | **B** | cargo clippy is a Rust linter that catches common mistakes and suggests improvements |
| Q11 | **B** | cargo check --all-targets compiles the code without producing binaries, saving time |
| Q12 | **C** | --all-targets compiles all targets including benchmarks and examples, ensuring everything builds |
| Q13 | **A** | Integration tests are separate from unit tests and verify components work together |
| Q14 | **D** | The hook aborts the commit and shows which files need formatting with cargo fmt --all |
| Q15 | **C** | cargo doc --document-private-items compiles documentation and checks for warnings |
| Q16 | **A** | Running cargo check first saves time by failing fast on compilation errors |
| Q17 | **B** | Pre-commit typically takes 30-60 seconds depending on project size and tests |
| Q18 | **D** | Release mode tests are only run in the pre-push hook, not pre-commit |

### Part 3: Pre-push Hook Details

| Question | Correct Answer | Explanation |
|----------|----------------|-------------|
| Q19 | **B** | Release optimizations can introduce subtle bugs, so we verify compilation in release mode |
| Q20 | **C** | Pre-commit checks basic formatting and linting, pre-push adds release mode and extra validations |
| Q21 | **A** | git diff --cached --name-only shows staged files; we use this to check for target/ |
| Q22 | **B** | Release optimizations can change floating point behavior, eliminate dead code, or expose memory issues |
| Q23 | **C** | cargo-tarpaulin is the standard tool for measuring Rust code coverage |
| Q24 | **A** | Code coverage is informational - the hook runs it but doesn't block the push if below threshold |
| Q25 | **A** | The hook verifies Cargo.lock is up to date (changes should be committed together) |
| Q26 | **C** | TODO/FIXME comments indicate incomplete work - checking them encourages cleanup before pushing |
| Q27 | **D** | The hook extracts Rust code blocks from README and verifies they exist and look syntactically valid |
| Q28 | **D** | All of the above - release mode tests, generated files, coverage, examples, documentation |

### Part 4: Troubleshooting & Best Practices

| Question | Correct Answer | Explanation |
|----------|----------------|-------------|
| Q29 | **D** | Check all three: file existence, executable permission, and hooksPath configuration |
| Q30 | **A** | The hook file must have execute permission (chmod +x) to run as a script |
| Q31 | **A** | git commit --no-verify bypasses the pre-commit hook for that commit |
| Q32 | **B** | Always fix the issue and commit again - bypassing defeats the purpose of the hooks |
| Q33 | **A** | Faster hooks reduce friction and encourage more frequent, smaller commits |
| Q34 | **A** | Fast local feedback prevents broken code from reaching the repository |
| Q35 | **A** | git hook list shows which hooks are installed and their location |

---

## 🎯 Scoring and Feedback

### Calculate Your Score

```
Part 1 (Git Hooks Fundamentals):    _____ / 8
Part 2 (Pre-commit Details):         _____ / 10
Part 3 (Pre-push Details):           _____ / 10
Part 4 (Troubleshooting & Best):   _____ / 7
------------------------------------------------
TOTAL SCORE:                          _____ / 35
```

### Interpret Your Score

#### 30-35 Points: Excellent! 🎉
- You have a strong understanding of CI/CD pipelines
- Ready to set up CI/CD for other projects
- Could contribute to CI/CD best practices documentation

#### 25-29 Points: Good! 👍
- You understand most CI/CD concepts well
- Review hook details in Parts 2 and 3
- Understand when to use hooks vs GitHub Actions

#### 20-24 Points: Fair 👌
- You have basic understanding but gaps remain
- Review all parts systematically
- Practice by installing and using hooks

#### Below 20 Points: Needs Review 📚
- Return to CI Pipeline Setup Guide
- Study git hooks documentation
- Practice with test hooks on a sample project

### Recommended Next Steps

1. **Review Wrong Answers**: Go back to specific questions and understand why you were wrong
2. **Read Setup Guide**: Revisit `docs/ci-pipeline-setup.md` for detailed explanations
3. **Practice Locally**: Install hooks on a test repository and try triggering them
4. **Read Hook Scripts**: Examine `.githooks/pre-commit` and `.githooks/pre-push`
5. **Customize Hooks**: Add custom checks specific to your workflow
6. **Combine with GitHub Actions**: Learn when to use local hooks vs remote CI

---

## 📚 Study Resources

Based on your performance, focus on:

### Struggled with Git Hooks Fundamentals (Part 1)?
- Review [Git Hooks Documentation](https://git-scm.com/book/en/v2/Customizing-Git-Hooks.html)
- Try creating a simple test hook
- Read [Setup Guide](docs/ci-pipeline-setup.md) Chapter 1

### Struggled with Pre-commit Details (Part 2)?
- Review [Setup Guide](docs/ci-pipeline-setup.md) Chapter 3
- Run `cargo fmt`, `cargo clippy`, `cargo check` manually to understand them
- Read the actual hook script: `.githooks/pre-commit`

### Struggled with Pre-push Details (Part 3)?
- Review [Setup Guide](docs/ci-pipeline-setup.md) Chapter 4
- Understand why release mode tests are important
- Read the actual hook script: `.githooks/pre-push`

### Struggled with Troubleshooting (Part 4)?
- Review [Setup Guide](docs/ci-pipeline-setup.md) Chapters 8 and 9
- Practice bypassing hooks (and when it's appropriate)
- Read hook error messages carefully - they often contain hints

---

## 💡 Tips for Learning

1. **Read the Scripts**: The hook scripts are well-commented - read them to understand what they do
2. **Break Things**: Intentionally break formatting, add a clippy warning, and see how hooks react
3. **Measure Performance**: Time how long hooks take on your machine
4. **Customize**: Add checks that matter to your specific workflow
5. **Share with Team**: If working with others, agree on hook configuration
6. **Review Regularly**: Hook scripts evolve - review them periodically

---

## 🚀 Advanced Topics

Once you master the basics, consider exploring:

### Customizing for Your Workflow

```bash
# Add project-specific checks to pre-commit
check_project_standards() {
    print_header "Project Standards Check"
    
    # Check for TODO comments in src/ only (excluding docs/)
    SRC_TODOS=$(git diff --cached --name-only --diff-filter=AM src/ | xargs grep -h "TODO" 2>/dev/null | wc -l || echo "0")
    
    if [ "$SRC_TODOS" -gt 0 ]; then
        print_warning "Found $SRC_TODOS TODO(s) in source files"
    fi
}

# Check for benchmark results before pushing
check_benchmarks() {
    print_header "Benchmark Results Check"
    
    # Ensure benchmarks have been run
    if git diff --cached --name-only | grep -q "benches/"; then
        print_info "Benchmarks changed - ensure results are updated"
    fi
}
```

### Integrating with Team Workflows

- **Shared Hooks Repository**: Store hooks in a separate repo for team consistency
- **Hook Versioning**: Tag and version hooks for reproducibility
- **Team Review**: Regularly review and update hooks as team needs evolve
- **Documentation**: Maintain team-specific CI/CD documentation

### Beyond Local Hooks

When local hooks are not enough:
- **GitHub Actions**: For pull request validation, automated testing on multiple OSes
- **GitLab CI/CD**: Similar to GitHub Actions but with different configuration
- **Jenkins**: For complex enterprise workflows
- **TeamCity**: For continuous delivery and deployment automation

---

## 📖 Real-World Scenarios

### Scenario 1: Fast Iteration

**Situation**: You're rapidly prototyping a new feature

**Best Practice**:
- Temporarily disable expensive checks (like release mode tests)
- Keep formatting and basic compilation checks enabled
- Re-enable all checks before pushing

```bash
# For development commits
git commit --no-verify -m "WIP: Prototype feature"

# Before pushing, ensure all checks pass
git add .
git commit -m "Feature complete"
git push
```

### Scenario 2: Team Collaboration

**Situation**: Working on a team project with agreed coding standards

**Best Practice**:
- Use shared hooks repository
- Ensure all team members have identical hook versions
- Document team-specific hook configurations
- Review hook failures in code reviews

### Scenario 3: Multiple Projects

**Situation**: Working on several Rust projects simultaneously

**Best Practice**:
- Each project has its own hooks (tailored to project needs)
- Shared tools repository for common hook utilities
- Personal global hooks for universal checks (like formatting)
- Project-specific hooks in each repository

### Scenario 4: Emergency Fixes

**Situation**: Critical bug found in production, need to deploy fix immediately

**Best Practice**:
```bash
# 1. Make the fix
# 2. Test thoroughly locally
# 3. Bypass hooks only for the emergency fix
git commit --no-verify -m "HOTFIX: Critical production issue"
# 4. Push
git push --no-verify
# 5. Immediately fix the root cause and restore hooks
```

---

**This assessment is designed to test your understanding of local CI/CD pipelines and help you identify areas for improvement. Use it as a learning tool to master automation in your development workflow!** 🚀