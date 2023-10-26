# list

## Description

List workspaces.

## Usage

```bash
tfct workspace list [options]
```

## Options

| Short | Long       | Description                                      |
|-------|------------|--------------------------------------------------|
| `-f`  | `--filter` | Filter the list of workspaces by given criteria. |

## Examples

### List all workspaces

```bash
tfct workspace list
```

### Get a list of workspaces with wildcard search

```bash
tfct workspace list -f --query-wildcard-name "my-*"
```

### List all workspaces and save output to a file

```bash
tfct workspace list --save-output --output workspaces.json
```