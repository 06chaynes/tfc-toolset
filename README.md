# Terraform Cloud Toolset

A collection of tools to help in management of your Terraform cloud organization.

## What's in the box?

- Wrath.

- `tfc-toolset`: The core library containing common functions for working with the Terraform cloud API. Each tool extends upon this library to accomplish its specific purpose.
  
- `which-workspace`: A search tool for finding workspaces that match specified parameters. Sometimes you just need to know which workspaces have a specific value for a specific variable right? ... Right? Happens all the time I'm sure.
  
- `clean-workspace`: Throw your workspaces in a tub and scrub! Generate massive overly verbose reports on your hundreds of workspaces with details of all the things you need to fix! Maybe even fix some of those things automagically (or maybe not!)!

## Why?

Because I like a side of automation with my automation. nom nom nom

## How do I use this?

More details on each tool/library can be found on the README in each project directory (as long as future me actually finished them, that guy needs to step it up), links below:

- [`tfc-toolset`](tfc-toolset/README.md)
  
- [`which-workspace`](which-workspace/README.md)

- [`clean-workspace`](clean-workspace/README.md)

See [Filtering](FILTERING.md) for more details on the specifics of workspace filtering.
