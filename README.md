# lr2-launcher (working title)

> Experimental launcher for LR2 / [OpenLR2](https://github.com/GOMazk/OpenLR2)

Automated builds can be found in the [Actions](https://github.com/SayakaIsBaka/lr2-launcher/actions) tab (requires a GitHub account) or on [nightly.link](https://nightly.link/SayakaIsBaka/lr2-launcher/workflows/rust/master?preview); **use at your own risk!**

It is **HIGHLY** recommended you perform a backup of your LR2 and OpenLR2 configuration (`LR2files\Config\config.xml` and `LR2files\Config\openlr2-config.xml`) as well as your player databases (`LR2files\Database\Score` folder) before using this! While the player databases are only opened in read-only mode, you never know what can happen.

# What's missing

- Translations (it's in the GUI but it doesn't do anything yet, also I don't have anyone to translate the strings)
- Course file (`.lr2crs` files) add
- Disable score save switch does not actually work (the `-ns` argument only works when a chart is directly loaded apparently)
- File / folder drag and drop support (waiting for Slint to support it which should hopefully be [soon](https://github.com/slint-ui/slint/issues/1967))
- Other things probably (obscure LR2 options / features no one uses for example)

# Credits

- The [LR2Nexus](https://github.com/Unengine/LR2Nexus) project for the general UI layout, figuring out some stuff with the config format and in general for giving me the motivation to work on this
- The [OpenLR2](https://github.com/GOMazk/OpenLR2) team for their amazing work to keep LR2 alive and their help on the project