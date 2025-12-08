# Simple Web Server

Inspired by https://x.com/davepl1968/status/1995882406709264689?t=BODFF_Nk5wBhQ7aZY4qiMQ&s=19, build a simple web server example. Only crate support is Tokio for async and Tracing for logging.

## Notes

* Not using BufStream because in general we do large reads / writes

## Resources

* https://tokio.rs/tokio/tutorial/
* https://dev.to/geoffreycopin/-build-a-web-server-with-rust-and-tokio-part-0-the-simplest-possible-get-handler-1lhi
