# ytextract - A library for getting YouTube metadata

[![Github](https://img.shields.io/badge/github-ATiltedTree/ytextract-8da0cb?style=for-the-badge&labelColor=555555&logo=github)](https://github.com/ATiltedTree/ytextract)
[![Crates.io](https://img.shields.io/crates/v/ytextract?style=for-the-badge&logo=rust)](https://crates.io/crates/ytextract)
[![docs.rs](https://img.shields.io/docsrs/ytextract?color=teal&style=for-the-badge)](https://docs.rs/ytextract)
[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/ATiltedTree/ytextract/Test?style=for-the-badge)](https://github.com/ATiltedTree/ytextract/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/ATiltedTree/ytextract/branch/main/graph/badge.svg?token=6CFXYPTNV7)](https://codecov.io/gh/ATiltedTree/ytextract)

---

This includes:

- [x] Videos
- [x] Streams (e.g. downloading of videos)
- [x] Playlists
- [x] Channels
- [ ] Community Posts
- [ ] Comments
- [ ] Closed Captions
- [ ] Search
- [ ] Live Streams

## Basic Example

```rust
#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get a Client for making request
    let client = ytextract::Client::new().await?;

    // Get information about the Video identified by the id "nI2e-J6fsuk".
    let video = client.video("nI2e-J6fsuk".parse()?).await?;

    // Print the title of the Video
    println!("Title: {}", video.title());

    Ok(())
}
```

## Notes

- ### Compiler support

    This library always expects to be used with the latest version of rust. It
    may run on older rust versions, but not guarantee is made, that it won't
    break between versions.

- ### Subscriber count

    All functions that return subscriber counts only return 3-digit precision
    values as that is all that YouTube returns. That means if channel has
    exactly `164_583` subscribers, this library will return `164_000`.

- ### Panic behavior

    This library should never panic. If it does, it should be reported as a
    bug. Panics mostly mean, that YouTube changed something that this library
    could not deal with.
