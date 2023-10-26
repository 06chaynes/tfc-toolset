# list

## Description

List the variables for a workspace.

## Usage

```bash
tfct variable list [options]
```

## Examples

### List the variables for a workspace

```bash
tfct variable list --workspace-name "my-workspace"
```

### List the variables for a workspace and save output to a file

```bash
tfct variable list --workspace-id "ws-id" --save-output --output variables.json
```