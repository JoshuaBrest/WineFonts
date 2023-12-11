# Docs

This doc refers to the format of the JSON file accessible at `https://winefonts.bashed.sh/fonts.json`.

## Version

This should be self-explanatory. It is the version of the JSON file.

## Groups

This is an array of objects. Each object represents a group of fonts. The object has the following properties:

* `name` This is the name of the group.
* `fonts` This is an array of font ids. (See [Fonts](#fonts))

## Downloads

This is an array of objects. Each object represents a downloadable file. The object has the following properties:

* `id`: This is the UUID of the download. It is used to identify the download. (This changes)
* `fileSize`: This is the size of the file in bytes.
* `hash`: This is the sha256 hash of the file.
* `downloadURL`: This is the URL to download the file.

## Fonts

This is an array of objects. Each object represents a font. The object has the following properties:

* `id`: This is the UUID of the font. It is used to identify the font. (It cannot be changed throught versions)
* `name`: This is the name of the font.
* `shortName`: This is the short name of the font.
* `publisher`: This is the publisher of the font. 
* `categories`: This is an array of categories that the font belongs to. (See [Categories](#categories))
* `installations`: This is an array of objects. Each object represents a **REQUIRED** method to fully install a font.

### Categories

These are just font categories like `sans-serif`.

Here are the categories:
* `sans-serif`
* `serif`
* `monospace`
* `cursive`

### Installations

There are many types of installations denoted by the `type` property. Each type has its own properties but they all have the following properties:

* `type`: This is the type of installation.
* `download`: This is the download URL. (See [Download](#download))

Types:
* `cabextract`: See [Cabextract](#cabextract)

#### Cabextract

This is the type for fonts that are distributed in a cab file. It has the following properties:

* `files`: The files to extract from the exe file and install.