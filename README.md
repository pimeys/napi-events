# N-API tracing test

A test project to demonstrate a possible way to stream logs from a Rust N-API
module to a JavaScript listener.

## Building

``` sh
> cargo build --release
// Linux
> cp target/release/libnapi_events.so libnapi_events.so.node
// macOS
> cp target/release/libnapi_events.dylib libnapi_events.dylib.node
// Windows
> cp target/release/napi_events.dll napi_events.dll.node
```

Edit `index.js` to load the correct file (defaults to `.so` for Linux). Then run
`node index.js` to see two JSON-formatted events printed to the console
