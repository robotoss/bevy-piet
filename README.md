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
bevy = { path = "bevy", default-features = false }
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
