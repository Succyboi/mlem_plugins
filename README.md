# Mlem Plugins

## Includes

- [mlem_meter](mlem_meter) 0.1.1 - An audio meter.
    > Measures input according to the [LUFS](https://en.wikipedia.org/wiki/LUFS) standard.
    > 
    > ![mlem_meter](mlem_meter/preview.png)
- [lua_garden_plug](lua_garden_plug) - Total WIP

## Building

Current platform example:

```
cargo xtask bundle mlem_meter --release
```

For windows:
```
cargo xtask bundle mlem_meter --release --target x86_64-pc-windows-gnu
```