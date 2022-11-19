<div align="center">

# 🎨📐 bevy-piet

**A plugin that integrates piet-gpu into bevy as an alternative renderer**

</div>

## Usage
Add plugin to your bevy project (`default features = false` to disable bevy's default renderer):

**NOTE**: you may need to add the `"xll"` feature to bevy if running on Linux.

```
[dependencies]
bevy-piet = { path = "bevy-piet" }
bevy = {version ="0.9", default-features = false }
```

Put in `main()`:
```
use bevy::prelude::*;
use bevy_piet::BevyPietPlugins;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyPietPlugins)
        .run();
```

# License
This project is dual-licensed under [Apache 2.0](https://github.com/Seabass247/bevy-piet/blob/main/LICENSE-APACHE) and [MIT](https://github.com/Seabass247/bevy-piet/blob/main/LICENSE-MIT).
