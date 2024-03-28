# Configuration

`tfct` is configured using either a configuration file or via cli arguments.

## TOML configuration file

The configuration file is a TOML file that can contain the following properties:

```toml
token = "tfc-access-token" # The Terraform Cloud API token to use when making requests.
org = "org-name" # The Terraform Cloud organization to use when making requests.
project = "project-id" # The project to use when making requests.
output = "production.json" # The location where report output should be written
save_output = true # Whether to save the report output of commands to the `output` location.
log = "info" # The log level to use when logging messages. Valid values are `trace`, `debug`, `info`, `warn`, and `error`.

[workspaces.query]
name = "aws-" # The name of the workspace to fuzzy search for
wildcard_name = "*-prod" # The wildcard name of the workspace to search for

[[workspaces.query.tags]] # The tag to search for, formatted as operator:name
operator = "NotContains"
name = "team:"

[[workspaces.query.variables]] # The variable to search for, formatted as key:operator:value
key = "mode"
operator = "Contains"
value = "prod"

[pagination]
start_page = "1" # The page to start at when retrieving data with default of `1` (first page)
max_depth = "1" # The maximum number of pages to retrieve with default of `1` (first page only), 0 for all
page_size = "20" # The number of items to retrieve per page with default of `20` (20 items per page)
```

## CLI arguments

`tfct` can also be configured using cli arguments. The cli arguments are the similar as the properties in the configuration file,
but are prefixed with `--` and use `kebab-case` instead of `snake_case`.
For example, the `save_output` property in the configuration file would be `--save-output` as a cli argument.

```bash
--org <ORG>
  The name of the organization

--token <TOKEN>
  The token to use for authentication

--project <PROJECT>
  The id of the project

--log <LOG>
  The log level to use

--output <OUTPUT>
  The location where report output should be written

--start-page <START_PAGE>
  The page to start at when retrieving data

--max-pages <MAX_PAGES>
  The maximum number of pages to retrieve

--page-size <PAGE_SIZE>
  The number of items to retrieve per page

--save-output
  Save the output of the command to a file

--pretty-output
  Pretty print the output when saving to a file

--query-name <QUERY_NAME>
  The name of the workspace to fuzzy search for

--query-wildcard-name <QUERY_WILDCARD_NAME>
  The name of the workspace to wildcard search for

--query-variable <QUERY_VARIABLE>
  The name of the variable to search for, formatted as key:operator:value

--query-tag <QUERY_TAG>
  The name of the tag to search for, formatted as operator:name
```

## Valid Operators

The valid operators to use in the toml configuration file are:

- `Contains`
- `NotContains`
- `Equals`
- `NotEquals`

The valid operators to use in the cli arguments are:

- `~=` for `Contains`
- `!~=` for `NotContains`
- `==` for `Equals`
- `!=` for `NotEquals`

For more information on filtering see [Filtering](./filtering.md).
