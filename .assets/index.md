A collection of tools to help in management of your Terraform cloud organization.

## What's in the box?

- Wrath.

- `tfc-toolset`: The core library containing common functions for working with the Terraform cloud API. Each tool extends upon this library to accomplish its specific purpose.

- `tfc-toolset-extras`: Extends the core library with more optional functionality.

- `tfc-which-workspace`: A search tool for finding workspaces that match specified parameters. Sometimes you just need to know which workspaces have a specific value for a specific variable right? ... Right? Happens all the time I'm sure.

- `tfc-clean-workspace`: Throw your workspaces in a tub and scrub! Generate massive overly verbose reports on your hundreds of workspaces with details of all the things you need to fix! Maybe even fix some of those things automagically (or maybe not!)!

- `tfc-run-workspace`: A tool for creating multiple Terraform Cloud runs in parallel with provided specifications.

- `tfc-report-tui`: Reading large JSON payloads can be a headache, this maybe slightly less so.

## Why?

Because I like a side of automation with my automation. nom nom nom

## How do I use this?

More details on each tool/library can be found on the respective page below: