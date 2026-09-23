# Node 24 runs the TypeScript directly

**Status:** accepted

## Context

The engine is TypeScript. The usual answer is tsc, esbuild or tsx.

## Decision

Node 24 executes TypeScript natively. That removes a build step, a config file and a class of source-map problems, and it means a contributor needs only Node and Typst.

## Consequences

Type-stripping only: no enums, no parameter properties, no decorators. tsconfig sets erasableSyntaxOnly so the typechecker enforces it — which it did, the first time we wrote a class with a parameter property.
