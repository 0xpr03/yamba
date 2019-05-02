### yamba types
Contains types used by the daemon api and hared types of yamba-daemon and voip plugin code.

#### Generating docs
Run `cargo doc --no-deps --open`.  
To also get ts3plugin type docs use `cargo doc --no-deps --open --features rpc`

#### Modules
- `models` Main docs for API related things, contains callback section.
- `rpc` RPC things for voip plugins like ts3plugin.
- `track` Internal API types from yamba-daemon