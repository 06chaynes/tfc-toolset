# tfct

A tool to help manage a toolset that helps manage your deployments.

## Usage

```shell
tfct [OPTIONS] <COMMAND>
```

## [Configuration](./configuration/configuration.md)

Details on how to configure tfct.

## [Commands](./commands/commands.md)

Details on each of the commands available in tfct.

## Root Options
```shell
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

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```