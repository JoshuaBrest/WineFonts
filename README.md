# WineFonts

These fonts are a collection of windows fonts that are often used by windows applications. They are also organized in groups much like `winetricks`'s `corefonts` command.

## Usage

To use the uploaded version of the fonts, you can use the following link: `https://winefonts.bashed.sh/versions.json`. This link will always point to the latest version of the fonts.

<!-- See [docs.md](docs.md) for the format of the JSON file. -->

## Included Fonts

* Andale Mono v2.00
* Arial v2.82
* Arial Black v2.35
* Comic Sans MS v2.10
* Courier New v2.82
* Georgia v2.05
* Impact v2.35
* Times New Roman v2.82
* Trebuchet MS v1.22
* Verdana v2.35
* Webdings (unknown, probably 1.03)

## Notes

Please make sure when calling `cabextract` to use `-L` or `--lowercase` to ensure that the fonts are extracted with lowercase names. This is because not all filesystems are case sensitive.

## Contributing

When making a pull request, please do the following:
1. Do not change any existing uuids. If you need a new uuid, set it to `<UUID>` then run format.
2. If downloading a font from a website, do not use sketchy sources or expireable links. Some good sources are: [GitHub](https://github.com) and [The Internet Archive](https://archive.org).
3. Make sure to lint the JSON. This will require you to have [pnpm](https://pnpm.js.org) and [nodejs](https://nodejs.org) installed.
   * Run `pnpm --prefix utils install` to install the dependencies.
   * Run `pnpm --prefix utils run script:format` to format the JSON.
4. Make sure to check that it builds
  * Step 3 is required for this step.
  * Run `pnpm --prefix utils run script:build` to build the JSON.

## License

Local (non-redistributed) files including but not limited to this README and `fonts.json` are licensed under the LGPL-3.0-or-later.

## Categories

<details><summary>Core Fonts</summary>

* Andale Mono v2.00
* Arial v2.82
* Arial Black v2.35
* Comic Sans MS v2.10
* Courier New v2.82
* Georgia v2.05
* Impact v2.35
* Times New Roman v2.82
* Trebuchet MS v1.22
* Verdana v2.35
* Webdings (unknown, probably 1.03)

</details>