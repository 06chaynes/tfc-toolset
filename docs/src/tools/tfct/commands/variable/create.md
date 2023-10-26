# create

## Description

Create variables on a workspace.

## Usage

```bash
tfct variable create [options]
```

## Options

| Short | Long                    | Description                                                                                               |
|-------|-------------------------|-----------------------------------------------------------------------------------------------------------|
| `-v`  | `--var <VAR>`           | The variable to create on the workspace, in the format of 'key=value:description:category:hcl:sensitive'. |
|       | `--var-file <VAR_FILE>` | The file containing variables.                                                                            |

## Examples

### Create a variable on a workspace

```bash
tfct variable create --workspace-name "my-workspace" --var "SECRET_KEY=mysecretohno:ENV var for the secret:env:false:true"
```

```bash
tfct variable create --workspace--name "my-workspace" --var "newthing=vale::::true"
```

```bash
tfct variable create --workspace-name "my-workspace" --var "emptyvar="
```

### Create a variable on a workspace using a file
```json
{
  "variables": [
    {
      "var": "SECRET_KEY=mysecretohno:ENV var for the secret:env:false:true"
    },
    {
      "var": "newthing=vale::::true"
    }
  ]
}
```

```bash
tfct variable create --workspace-id "ws-id" --var-file vars.json
```
