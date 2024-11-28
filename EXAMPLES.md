# GitHub DB CLI Examples

This document provides examples of using the GitHub DB CLI tool with certificates for various operations.

## Certificate Management

First, generate a certificate for authentication:

```bash
# Generate a new certificate for user "alice"
github-db --path /path/to/db generate-cert alice --output ./certs

# List all valid certificates
github-db --path /path/to/db list-certs

# Revoke a certificate
github-db --path /path/to/db revoke-cert alice
```

## Basic CRUD Operations

For all data operations, you need to provide a valid certificate. You can do this in two ways:

```bash
# Using certificate file
github-db --cert ./certs/alice.cert --path /path/to/db <command>

# Using base64 encoded certificate content
github-db --cert-content $(base64 -i ./certs/alice.cert) --path /path/to/db <command>
```

### Create Documents

```bash
# Create a document with inline JSON
github-db --cert ./certs/alice.cert create user123 '{"name": "Alice", "age": 30}'

# Create a document using stdin
cat data.json | github-db --cert ./certs/alice.cert --stdin create user123

# Create with encryption
github-db --cert ./certs/alice.cert --key "mysecretkey" create user123 '{"name": "Alice", "age": 30}'
```

### Read Documents

```bash
# Read a document
github-db --cert ./certs/alice.cert read user123

# Read with JSON output format
DB_JSON_OUTPUT=1 github-db --cert ./certs/alice.cert read user123

# Read an encrypted document
github-db --cert ./certs/alice.cert --key "mysecretkey" read user123
```

### Update Documents

```bash
# Update a document with inline JSON
github-db --cert ./certs/alice.cert update user123 '{"name": "Alice", "age": 31}'

# Update using stdin
cat updated_data.json | github-db --cert ./certs/alice.cert --stdin update user123

# Update encrypted document
github-db --cert ./certs/alice.cert --key "mysecretkey" update user123 '{"name": "Alice", "age": 31}'
```

### Delete Documents

```bash
# Delete a document
github-db --cert ./certs/alice.cert delete user123

# Delete an encrypted document
github-db --cert ./certs/alice.cert --key "mysecretkey" delete user123
```

### List Documents

```bash
# List all documents
github-db --cert ./certs/alice.cert list

# List with JSON output
DB_JSON_OUTPUT=1 github-db --cert ./certs/alice.cert list
```

## Finding Documents with Filters

The `find` command allows you to search for documents using various filter conditions. Filters can be simple conditions or complex combinations using AND/OR logic.

### Simple Condition Filters

```bash
# Find users named "Alice"
github-db --cert ./certs/alice.cert find '{
  "type": "condition",
  "field": "name",
  "op": "eq",
  "value": "Alice"
}'

# Find users older than 25
github-db --cert ./certs/alice.cert find '{
  "type": "condition",
  "field": "age",
  "op": "gt",
  "value": 25
}'

# Find users in cities containing "York"
github-db --cert ./certs/alice.cert find '{
  "type": "condition",
  "field": "city",
  "op": "contains",
  "value": "York"
}'
```

### Complex AND/OR Filters

```bash
# Find users older than 25 AND living in New York
github-db --cert ./certs/alice.cert find '{
  "type": "and",
  "conditions": [
    {
      "type": "condition",
      "field": "age",
      "op": "gt",
      "value": 25
    },
    {
      "type": "condition",
      "field": "city",
      "op": "eq",
      "value": "New York"
    }
  ]
}'

# Find users who are either developers OR designers
github-db --cert ./certs/alice.cert find '{
  "type": "or",
  "conditions": [
    {
      "type": "condition",
      "field": "role",
      "op": "eq",
      "value": "developer"
    },
    {
      "type": "condition",
      "field": "role",
      "op": "eq",
      "value": "designer"
    }
  ]
}'
```

### Nested Field Filters

```bash
# Find users with a specific address city
github-db --cert ./certs/alice.cert find '{
  "type": "condition",
  "field": "address.city",
  "op": "eq",
  "value": "San Francisco"
}'

# Find users with active projects
github-db --cert ./certs/alice.cert find '{
  "type": "condition",
  "field": "projects.status",
  "op": "eq",
  "value": "active"
}'
```

### Filter Operators

The following operators are available for conditions:

- `eq`: Equal to
- `gt`: Greater than
- `lt`: Less than
- `gte`: Greater than or equal to
- `lte`: Less than or equal to
- `contains`: String contains (for string values)
- `startsWith`: String starts with (for string values)
- `endsWith`: String ends with (for string values)

### Using Filters with Stdin

You can also provide filters through stdin for more complex queries:

```bash
# Using a filter file
cat filter.json | github-db --cert ./certs/alice.cert --stdin find
```

## Using Environment Variables

You can use environment variables to avoid repeating common parameters:

```bash
# Set environment variables
export DB_PATH=/path/to/db
export DB_CERT=/path/to/certs/alice.cert
export DB_KEY=mysecretkey
export DB_JSON_OUTPUT=1

# Now commands can be shorter
github-db create user123 '{"name": "Alice"}'
github-db read user123
github-db list
github-db find '{"type": "condition", "field": "name", "op": "eq", "value": "Alice"}'
```

## Best Practices

1. Always keep certificates secure and never share them
2. Use environment variables in scripts to avoid exposing sensitive information
3. Use `--stdin` for large documents or when reading from files
4. Enable encryption for sensitive data using the `--key` parameter
5. Use `DB_JSON_OUTPUT=1` when parsing output programmatically
6. Structure complex filters in separate JSON files for better maintainability
7. Use nested field paths (e.g., "address.city") to filter on nested document structures
8. Combine multiple conditions with AND/OR operators for precise filtering
