---
description: Perform comprehensive code review for Rust and Vue code focusing on architecture, design principles, and code quality.
handoffs:
  - label: Review Architecture
    agent: rust-systems-architect
    prompt: Review the architectural design of the code
    send: true
  - label: Generate Improvement Plan
    agent: general-purpose
    prompt: Create an improvement plan based on review findings
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Outline

The text the user typed after `/deep-code-review` is the file path(s) or directory to review. If empty, review changed files in the current git branch.

### Execution Flow

1. **Identify Target Files**:
   - If `$ARGUMENTS` is provided:
     - Parse as file path(s) or directory
     - If directory: Find all `.rs`, `.vue`, `.ts`, `.tsx` files recursively
     - If file(s): Use specified files
   - If `$ARGUMENTS` is empty:
     - Run `git diff --name-only main...HEAD` to get changed files
     - Filter for `.rs`, `.vue`, `.ts`, `.tsx` extensions
   - If no files found: ERROR "No files to review"

2. **Read and Analyze Files**:
   - Read each target file completely
   - Parse file structure (imports, types, functions, components)
   - Identify file purpose and role in the system

3. **Perform Multi-Dimensional Review**:

   For **Rust files** (`.rs`), evaluate:

   a. **Architecture & Design** (Critical):
      - [ ] **Trait Design**: Are traits well-defined with clear contracts?
      - [ ] **Interface Clarity**: Are public APIs intuitive and well-documented?
      - [ ] **Extensibility**: Can functionality be extended without modification (Open-Closed Principle)?
      - [ ] **Separation of Concerns**: Are responsibilities properly separated?
      - [ ] **Dependency Management**: Are dependencies properly abstracted (Dependency Inversion)?
      - [ ] **Module Organization**: Is the module structure logical and scalable?

   b. **Rust Best Practices** (Critical):
      - [ ] **Ownership & Borrowing**: Proper use of ownership, avoiding unnecessary clones?
      - [ ] **Error Handling**: Using `Result<T, E>` appropriately, proper error propagation?
      - [ ] **Type Safety**: Leveraging type system (newtype pattern, phantom types)?
      - [ ] **Async Patterns**: Proper use of `async/await`, avoiding blocking in async context?
      - [ ] **Arc/Mutex Usage**: Correct concurrent patterns, avoiding deadlocks?
      - [ ] **Iterator Patterns**: Using iterator chains instead of manual loops where appropriate?
      - [ ] **Zero-Cost Abstractions**: Avoiding runtime overhead, using compile-time features?

   c. **Design Principles** (High Priority):
      - [ ] **KISS (Keep It Simple)**:
        - Is the code as simple as possible but no simpler?
        - Are complex solutions justified by real requirements?
        - Can the logic be simplified without losing functionality?
      - [ ] **DRY (Don't Repeat Yourself)**:
        - Is there duplicated logic that should be abstracted?
        - Are similar patterns repeated across multiple functions?
        - Should common functionality be extracted to utilities?
      - [ ] **YAGNI (You Aren't Gonna Need It)**:
        - Is there speculative code for future features?
        - Are there unused abstractions or over-engineered solutions?
        - Is every piece of code serving a current need?
      - [ ] **SOLID Principles**:
        - **Single Responsibility**: Does each struct/function have one reason to change?
        - **Open-Closed**: Can behavior be extended without modifying existing code?
        - **Liskov Substitution**: Do trait implementations honor contracts?
        - **Interface Segregation**: Are traits focused and cohesive?
        - **Dependency Inversion**: Does code depend on abstractions, not concretions?

   d. **Builder Pattern Usage** (Medium Priority):
      - [ ] **When Appropriate**: Used for complex object construction with many optional fields?
      - [ ] **Type-State Pattern**: Using phantom types for compile-time state validation?
      - [ ] **Method Chaining**: Fluent API design for better ergonomics?
      - [ ] **Validation**: Builder validates state before constructing final object?

   e. **Code Quality** (High Priority):
      - [ ] **Naming**: Clear, descriptive names following Rust conventions?
      - [ ] **Documentation**: Public APIs documented with examples?
      - [ ] **Error Messages**: Actionable error messages for users?
      - [ ] **Testing**: Unit tests present and meaningful?
      - [ ] **Unsafe Usage**: If present, is it justified and documented?
      - [ ] **Clippy Compliance**: Would pass clippy with recommended lints?

   For **Vue/TypeScript files** (`.vue`, `.ts`, `.tsx`), evaluate:

   a. **Component Design** (Critical):
      - [ ] **Single Responsibility**: Does component have one clear purpose?
      - [ ] **Reusability**: Is component generic enough to be reused?
      - [ ] **Props Interface**: Clear, typed props with validation?
      - [ ] **Event Handling**: Proper event emission and handling?
      - [ ] **Composition API**: Using composition API patterns effectively?

   b. **Type Safety** (High Priority):
      - [ ] **TypeScript Usage**: Proper type annotations, avoiding `any`?
      - [ ] **Interface Definitions**: Clear interfaces for props, state, API responses?
      - [ ] **Type Guards**: Using type narrowing correctly?

   c. **State Management** (High Priority):
      - [ ] **State Location**: State at appropriate level (local vs global)?
      - [ ] **Reactivity**: Proper use of reactive primitives (ref, reactive)?
      - [ ] **Side Effects**: Effects properly managed with lifecycle hooks?

   d. **Design Principles** (High Priority):
      - [ ] **KISS**: Simple component logic, avoiding over-abstraction?
      - [ ] **DRY**: Composables/utilities for shared logic?
      - [ ] **YAGNI**: No speculative features or unused code?

   e. **Code Quality** (Medium Priority):
      - [ ] **Naming**: Clear component and variable names?
      - [ ] **Formatting**: Consistent code style?
      - [ ] **Comments**: Complex logic explained?
      - [ ] **Testing**: Component tests present?

4. **Generate Review Report**:

   Create a structured report in markdown format:

   ```markdown
   # Code Review Report

   **Date**: [Current Date]
   **Reviewer**: Claude Code (Automated Review)
   **Files Reviewed**: [Count] files ([Count] Rust, [Count] Vue/TS)

   ## Executive Summary

   [2-3 sentence overview of code quality, highlighting major strengths and concerns]

   **Overall Score**: [X/10]
   - Architecture & Design: [X/10]
   - Code Quality: [X/10]
   - Best Practices: [X/10]
   - Maintainability: [X/10]

   ---

   ## Critical Issues 🔴

   [List issues that MUST be addressed before merging]

   ### Issue 1: [Title]
   **File**: `path/to/file.rs:line`
   **Severity**: Critical
   **Category**: [Architecture/Security/Performance/etc.]

   **Problem**:
   [Describe the issue clearly]

   **Code**:
   ```rust
   [Show problematic code]
   ```

   **Why This Matters**:
   [Explain impact on system/users]

   **Recommendation**:
   [Specific, actionable fix]

   **Better Approach**:
   ```rust
   [Show improved code example]
   ```

   ---

   ## High Priority Issues 🟡

   [Issues that should be addressed soon but not blocking]

   ### Issue [N]: [Title]
   [Same format as Critical Issues]

   ---

   ## Improvements & Suggestions 🟢

   [Nice-to-have improvements that would enhance code quality]

   ### Suggestion [N]: [Title]
   **File**: `path/to/file.rs:line`
   **Category**: [Performance/Readability/etc.]

   **Current Implementation**:
   [Brief description or code snippet]

   **Suggestion**:
   [How to improve]

   **Benefits**:
   - [Benefit 1]
   - [Benefit 2]

   ---

   ## Strengths ✅

   [Highlight what the code does well]

   - **[Strength Category]**: [Description with file references]
   - **[Another Strength]**: [Description]

   ---

   ## Architecture Analysis

   ### Current Architecture
   [Describe the architectural patterns observed]

   ### Strengths
   - [Architectural strength 1]
   - [Architectural strength 2]

   ### Concerns
   - [Architectural concern 1]
   - [Architectural concern 2]

   ### Recommendations
   - [Recommendation 1]
   - [Recommendation 2]

   ---

   ## Design Principles Compliance

   ### KISS (Keep It Simple, Stupid)
   **Score**: [X/10]
   [Analysis of code simplicity]

   ### DRY (Don't Repeat Yourself)
   **Score**: [X/10]
   [Analysis of code duplication]

   ### YAGNI (You Aren't Gonna Need It)
   **Score**: [X/10]
   [Analysis of speculative code]

   ### SOLID Principles
   **Score**: [X/10]
   [Analysis of SOLID compliance]

   ---

   ## Builder Pattern Usage

   [If builders found in code]

   **Instances Found**: [Count]

   ### Analysis
   [Evaluate builder implementations]

   ### Recommendations
   [Suggest improvements or additional use cases]

   ---

   ## Rust-Specific Observations

   ### Ownership & Borrowing
   [Analysis of ownership patterns]

   ### Error Handling
   [Analysis of error handling strategies]

   ### Async Patterns
   [Analysis of async/await usage]

   ### Type System Usage
   [Analysis of type system leverage]

   ---

   ## Vue/TypeScript Observations

   [If Vue files reviewed]

   ### Component Design
   [Analysis of component structure]

   ### Type Safety
   [Analysis of TypeScript usage]

   ### State Management
   [Analysis of state patterns]

   ---

   ## Testing Coverage

   [Analysis of test presence and quality]

   ---

   ## Action Items

   ### Must Do (Before Merge)
   - [ ] [Action 1] - `file.rs:line`
   - [ ] [Action 2] - `file.vue:line`

   ### Should Do (Next Sprint)
   - [ ] [Action 3]
   - [ ] [Action 4]

   ### Consider (Future)
   - [ ] [Action 5]
   - [ ] [Action 6]

   ---

   ## References

   - [Rust 编码设计原则](../rust编码设计原则.md)
   - [Project CLAUDE.md](../CLAUDE.md)
   - [Project Constitution](../.specify/memory/constitution.md)
   ```

5. **Save Review Report**:
   - Generate filename: `code-review-[YYYY-MM-DD]-[branch-name].md`
   - Save to `.claude/reviews/` directory (create if needed)
   - Print report to console
   - Provide summary with key findings count

6. **Offer Follow-up Actions**:

   After presenting the report, offer:

   ```markdown
   ## Next Steps

   I've completed the code review. Here are your options:

   1. **Review Architecture** - Have rust-systems-architect agent do deeper architectural analysis
   2. **Generate Improvement Plan** - Create a prioritized plan to address findings
   3. **Auto-Fix Simple Issues** - Let me fix straightforward issues automatically
   4. **Explain Specific Issue** - Ask about any finding in detail
   5. **Review Another File** - Run review on different files

   What would you like to do?
   ```

---

## General Guidelines

### Review Philosophy

- **Be Constructive**: Focus on improvement, not criticism
- **Be Specific**: Always reference exact file locations and line numbers
- **Be Actionable**: Every issue should have a clear path to resolution
- **Be Balanced**: Acknowledge strengths as well as weaknesses
- **Be Educational**: Explain *why* something is an issue, not just *what*

### Scoring Rubric

**10/10 - Exceptional**: Production-ready, exemplary practices, well-tested, documented
**8-9/10 - Strong**: High quality, minor improvements possible
**6-7/10 - Good**: Solid code with some areas for improvement
**4-5/10 - Fair**: Functional but needs refactoring or has design issues
**2-3/10 - Poor**: Significant issues, needs major rework
**0-1/10 - Critical**: Fundamental problems, unsafe, or non-functional

### Issue Severity

- **Critical 🔴**: Security vulnerabilities, memory unsafety, data corruption, blocking bugs
- **High Priority 🟡**: Design flaws, performance issues, maintainability concerns
- **Improvements 🟢**: Optimization opportunities, readability enhancements, nice-to-haves

### Rust-Specific Focus Areas

When reviewing Rust code, pay special attention to:

1. **Memory Safety**: No unsafe code without justification, proper lifetime annotations
2. **Error Propagation**: Using `?` operator, avoiding `unwrap()` in production code
3. **Async Correctness**: No blocking operations in async functions, proper future handling
4. **Performance**: Avoiding unnecessary allocations, using iterators, zero-copy when possible
5. **API Design**: Ergonomic APIs following Rust conventions and guidelines
6. **Testing**: Unit tests for pure functions, integration tests for systems
7. **Documentation**: Examples in doc comments, explaining non-obvious behavior

### Vue-Specific Focus Areas

When reviewing Vue code:

1. **Composition API**: Prefer composition over options API for reusability
2. **TypeScript Integration**: Strong typing throughout components
3. **Performance**: Avoiding unnecessary reactivity, using computed properties
4. **Accessibility**: Proper ARIA labels, keyboard navigation
5. **Component Decomposition**: Breaking down large components into smaller ones
6. **Props Validation**: Runtime and compile-time validation

### Context Awareness

Before reviewing, always:
- Check `CLAUDE.md` for project-specific patterns and conventions
- Reference `rust编码设计原则.md` for Rust standards
- Review `.specify/memory/constitution.md` for project principles
- Understand the file's role in the larger system architecture

### Builder Pattern Evaluation

When evaluating builder patterns, check:
1. **Necessity**: Is the object complex enough to warrant a builder?
2. **Type Safety**: Using type-state pattern for compile-time validation?
3. **Ergonomics**: Is the API fluent and intuitive?
4. **Validation**: Are invalid states prevented at compile time?
5. **Documentation**: Clear examples of builder usage?

**Good Builder Example**:
```rust
// Type-state builder pattern
pub struct ConnectionBuilder<State = NeedHost> {
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    _state: PhantomData<State>,
}

// States
pub struct NeedHost;
pub struct NeedPort;
pub struct Ready;

impl ConnectionBuilder<NeedHost> {
    pub fn new() -> Self { /* ... */ }
    pub fn host(self, host: String) -> ConnectionBuilder<NeedPort> { /* ... */ }
}

impl ConnectionBuilder<NeedPort> {
    pub fn port(self, port: u16) -> ConnectionBuilder<Ready> { /* ... */ }
}

impl ConnectionBuilder<Ready> {
    pub fn database(mut self, db: String) -> Self { /* ... */ }
    pub fn build(self) -> Connection { /* ... */ }
}

// Usage enforces correct order at compile time:
let conn = ConnectionBuilder::new()
    .host("localhost".into())  // Must be first
    .port(5432)                // Must be second
    .database("mydb".into())   // Optional
    .build();                  // Can only call after required fields
```

### Anti-Patterns to Flag

**Rust**:
- Excessive `.clone()` calls
- `.unwrap()` in production code without justification
- Long functions (>100 lines)
- Deep nesting (>4 levels)
- God structs with too many responsibilities
- Blocking operations in async functions
- Mutex instead of RwLock for read-heavy workloads
- Missing error context (use `.context()` with anyhow)

**Vue/TypeScript**:
- Using `any` type
- Large component files (>300 lines)
- Direct DOM manipulation instead of Vue reactivity
- Missing key in v-for
- Not cleaning up side effects in onUnmounted
- Prop drilling (consider provide/inject)
- Mutating props directly

### Report Quality Standards

Every review report must:
1. Include specific file paths and line numbers
2. Provide code examples for issues and solutions
3. Explain the "why" behind each recommendation
4. Include actionable next steps
5. Acknowledge what the code does well
6. Be formatted in clean, readable markdown
7. Include a summary suitable for non-technical stakeholders

---

## Example Usage

```bash
# Review specific file
/deep-code-review backend/src/services/db_service.rs

# Review multiple files
/deep-code-review backend/src/services/*.rs

# Review directory
/deep-code-review backend/src/services/

# Review all changes in current branch (default)
/deep-code-review
```

---

## Output Location

Reports are saved to:
`.claude/reviews/code-review-[YYYY-MM-DD]-[branch-name].md`

This location is git-ignored but preserved locally for reference.
