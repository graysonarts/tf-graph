# tf-graph

Generate a `graphviz` digraph for a set of terraform projects that load remote backend data to determine the best order of operation.

To run

`tf-graph <path to root of all terraform files> | dot -Tpng -ooutput.png`

This will generate a png that looks something like this:

![./example.png](Parent <-- Child <--> Cycle)

## Supported Backends

- Local `tfstate` files
- S3 Backends
