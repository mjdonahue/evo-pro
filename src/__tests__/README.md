# Multi-Level Testing Strategy

This document outlines the multi-level testing strategy for the Evo Design project. The strategy includes unit tests, integration tests, end-to-end tests, and property-based tests, each serving a different purpose in ensuring the quality and reliability of the application.

## Testing Levels

### Unit Tests

Unit tests focus on testing individual components, functions, or classes in isolation. They verify that each unit of code works as expected without dependencies on other parts of the system.

**Characteristics:**
- Fast execution
- No external dependencies
- Test a single unit of code
- Use mocks for dependencies

**Location:** `src/__tests__/*.unit.test.ts`

**Example:** See `src/__tests__/utils.unit.test.ts` for an example of unit tests for utility functions.

**Running Unit Tests:**
```bash
pnpm test:unit
```

### Integration Tests

Integration tests verify that different parts of the application work together correctly. They test the interaction between components, services, and the API layer.

**Characteristics:**
- Test multiple units working together
- May include some external dependencies
- Focus on the boundaries between components
- Use more realistic mocks

**Location:** `src/__tests__/**/*.integration.test.ts`

**Example:** See `src/__tests__/integration/api.integration.test.ts` for an example of integration tests for the API layer.

**Running Integration Tests:**
```bash
pnpm test:integration
```

### End-to-End Tests

End-to-end tests verify that the entire application works as expected from the user's perspective. They simulate user interactions and verify that the application responds correctly.

**Characteristics:**
- Test the entire application
- Include all external dependencies
- Focus on user flows
- Run in a browser environment

**Location:** `e2e/*.spec.ts`

**Example:** See `e2e/app.spec.ts` for an example of end-to-end tests for the application.

**Running End-to-End Tests:**
```bash
pnpm test:e2e
```

### Property-Based Tests

Property-based tests verify that certain properties hold for all inputs within a specified domain. Instead of using specific examples, they generate many test cases automatically to find edge cases and unexpected behaviors.

**Characteristics:**
- Generate test cases automatically
- Focus on properties rather than examples
- Help find edge cases and unexpected behaviors
- Provide more thorough testing of input domains

**Location:** `src/__tests__/*.property.test.ts`

**Example:** See `src/__tests__/validation.property.test.ts` for an example of property-based tests for validation functions.

**Running Property-Based Tests:**
```bash
pnpm test:unit
```

Property-based tests are run as part of the unit tests since they test individual functions in isolation.

### Concurrent System Tests

Concurrent system tests verify that systems with concurrent operations behave correctly under various conditions. These tests use specialized utilities to create controlled and deterministic environments for testing concurrent code.

**Characteristics:**
- Test concurrent operations in a controlled environment
- Verify correct behavior under race conditions
- Test message ordering and error handling
- Provide deterministic results for non-deterministic code

**Location:** `src/__tests__/concurrent/*.concurrent.test.ts`

**Example:** See `src/__tests__/concurrent/actor.concurrent.test.ts` for an example of testing concurrent systems using the actor model.

**Running Concurrent System Tests:**
```bash
pnpm test:unit
```

Concurrent system tests are run as part of the unit tests but focus on the specific challenges of testing concurrent code.

### Test Data Generation

Test data generation utilities provide a way to create realistic test data for different types of entities. These utilities help create consistent, realistic test data that can be used across different tests.

**Characteristics:**
- Generate realistic test data using templates and random generation
- Allow overriding specific attributes for test scenarios
- Support generating related entities with consistent relationships
- Provide a consistent API for all entity types

**Location:** `src/__tests__/data/generators.ts`

**Example:** See `src/__tests__/data/generators.test.ts` for examples of using the test data generators.

**Using Test Data Generators:**
```typescript
// Generate a single entity
const user = userGenerator.one();

// Generate multiple entities
const tasks = taskGenerator.many(5);

// Generate an entity with specific attributes
const project = projectGenerator.with({
  name: 'Test Project',
  status: 'active',
});

// Generate related entities
const { users, projects, tasks, comments } = generateRelatedEntities(2);
```

## Test Configuration

### Unit and Integration Tests

Unit and integration tests use Vitest as the test runner. The configuration files are:

- Unit tests: `vitest.config.ts`
- Integration tests: `vitest.integration.config.ts`

### End-to-End Tests

End-to-end tests use Playwright as the test runner. The configuration file is `playwright.config.ts`.

## Best Practices

### When to Use Each Testing Level

- **Unit Tests:** Use for testing individual functions, components, or classes in isolation. Focus on edge cases and error handling.
- **Integration Tests:** Use for testing the interaction between different parts of the application, such as components and services.
- **End-to-End Tests:** Use for testing critical user flows and ensuring that the application works as expected from the user's perspective.

### Test Coverage

Aim for high test coverage at the unit and integration levels. End-to-end tests should cover critical user flows but don't need to be exhaustive.

### Test Organization

- Group tests by feature or module
- Use descriptive test names
- Follow the Arrange-Act-Assert pattern
- Keep tests independent and idempotent

## Running All Tests

To run all tests (unit, integration, and end-to-end):

```bash
pnpm test:all
```

This will run the tests in sequence: unit tests, then integration tests, then end-to-end tests.
