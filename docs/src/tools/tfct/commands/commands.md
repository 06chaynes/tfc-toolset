# Commands

## [workspace](./workspace/workspace.md)

Manage workspaces.

## [variable](./variable/variable.md)

Manage workspace variables.

## [variable-set](./variable-set/variable-set.md)

Manage variable sets.

## [tag](./tag/tag.md)

Manage workspace tags.

## [run](./run/run.md)

Manage workspace runs.

## [clean](./clean/clean.md)

Run cleanup operations.

## [help](./help.md)

Prints help message for a command.

# Global Options

| Short | Long                                          | Description                                                               |
|-------|-----------------------------------------------|---------------------------------------------------------------------------|
| `-h`  | `--help`                                      | Prints help information.                                                  |
| `-V`  | `--version`                                   | Prints version information.                                               |
|       | `--org <ORG>`                                 | The name organization to use.                                             |
|       | `--token <TOKEN>`                             | The token to use for authentication.                                      |
|       | `--project-id <PROJECT_ID>`                   | The ID of the project to use.                                             |
|       | `--log <LOG>`                                 | The log level to use.                                                     |
|       | `--output <OUTPUT>`                           | The location where output should be written.                              |
|       | `--start-page <START_PAGE>`                   | The page to start at when retrieving data.                                |
|       | `--page-size <PAGE_SIZE>`                     | The number of items to retrieve per page.                                 |
|       | `--max-pages <MAX_PAGES>`                     | The maximum number of pages to retrieve.                                  |
|       | `--save-output`                               | Save the output of the command to a file.                                 |
|       | `--pretty-output`                             | Pretty print the output when saving to a file.                            |
|       | `--query-name <QUERY_NAME>`                   | The name of the workspace to fuzzy search for.                            |
|       | `--query-wildcard-name <QUERY_WILDCARD_NAME>` | The name of the workspace to wildcard search for.                         |
|       | `--query-variable <QUERY_VARIABLE>`           | The name of the variable to search for, formatted as key:operator:value.  |
|       | `--query-tag <QUERY_TAG>`                     | The name of the tag to search for, formatted as operator:name.            |


