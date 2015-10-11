# rust-graphite-client
Sends HTTP request to graphite API, parses JSON response and aggregates ticks.

This is a typical a dry console Rust program. I developed it for Rust version 1.3

There are some issues I had to solve:

Command line arguments.
getopt doesn't support safe password enter, so I discovered rpassword.

Most of the time I fight with JSON deserialization (.as_f64()) 
Also I spent a lot of time with passing function pointers for aggregation function.

Hyper library is used to send HTTP request.
Basic HTTP authentication is the custom header which is not used in the library demo.
