<div align="center">
<img src="https://github.com/wanghoi/kwui-rs/raw/master/docs/icon.svg" height="140px" />

# kwui-cli

Command line helpers for [kwui-rs](https://github.com/wanghoi/kwui-rs) 

</div>

## Quick start

- Install
  ```bash
  cargo add kwui-cli
  ```
- Packaging resources
  ```bash
  # Pack 'assets' directory as kwui gui resources
  kwui pack-archive res.ar assets:/
  ```
- Parameters
  ```bash
  kwui --help
  kwui pack-archive --help
  kwui unpack-archive --help
  kwui list-archive --help
  ```

## Technical internals

- Resources indexed by 3-branch trie.
- Optional Lzf compression for non-images.