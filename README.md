# Arxy
Arxy is a forward proxy to help with making API calls to 3rd party services. It is not a gateway to aggregate multiple 3rd party services, although it isn't hard to add a GraphQL front end using something like Juniper (https://github.com/graphql-rust/juniper).

Arxy is built with Actix-web but it is simple enough to potentially be built with a low level http library.

## Getting started

1. Install via cargo

```sh
cargo install arxy
```

1. Start arxy

```sh
# start with defaults, just echos what was passed through
arxy

# common command line options, please see section on configuration below
arxy -p <port>
arxy -f <path to config>

```


## Configuration

### Command line options

```
  -p, --port
    Listen port. Default to `8080`

  -f, --config-file
    Path to config file. Default to `./arxy.config.json`
```

### Configuration file
A JSON file to configure both server and forwarded routes. see `arxy.config.json.example` for an example config.
