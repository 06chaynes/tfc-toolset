# spec

## Description

Queue up speculative plan runs.

## Usage

```bash
tfct run spec [options]
```

## Options

| Short | Long                                                                    | Description                                                                                                                      |
|-------|-------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------|
| `-w`  | `--workspace-name <WORKSPACE_NAME>`                                     | The name of the workspace to create the run on.                                                                                  |
| `-i`  | `--workspace-id <WORKSPACE_ID>`                                         | The id of the workspace to create the run on.                                                                                    |
| `-f`  | `--workspace-file <WORKSPACE_FILE>`                                     | The file containing a list of workspace names or IDs.                                                                            |
| `-a`  | `--auto-discover-workspaces`                                            | Automatically discover workspaces given the specified filters.                                                                   |
| `-q`  | `--queue <QUEUE>`                                                       | Execute runs in batches with overridable limits.                                                                                 |
|       | `--queue-max-concurrent <QUEUE_MAX_CONCURRENT>`                         | The maximum number of runs to execute concurrently.                                                                              |
|       | `--queue-max-iterations <QUEUE_MAX_ITERATIONS>`                         | The maximum number of times to check the status of a run before giving up.                                                       |
|       | `--queue-status-check-sleep-seconds <QUEUE_STATUS_CHECK_SLEEP_SECONDS>` | The number of seconds to wait between checking the status of a run.                                                              |
|       | `--message <MESSAGE>`                                                   | A message to include with the run [default: "Run created by tfc-toolset"].                                                       |
|       | `--target-addrs <TARGET_ADDRS>`                                         | A list of resource addresses to target for the run.                                                                              |
|       | `--replace-addrs <REPLACE_ADDRS>`                                       | A list of resource addresses to replace for the run.                                                                             |
|       | `--terraform-version <TERRAFORM_VERSION>`                               | The version of Terraform to use for this run, overriding the value from settings.                                                |

## Examples

### Create a run on a workspace

```bash
tfct run spec --workspace-name "my-workspace"
```

### Create a run on a multiple workspaces with a max concurrency of 2

```bash
tfct run spec --workspace-file "workspaces.json" -q --queue-max-concurrent 2
```
