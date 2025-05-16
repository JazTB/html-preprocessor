# html-preprocessor
Preprocesses HTML files to allow cross-site file-includes and similar conveniences without needing to touch javascript.

NOTE: Do not expect this to be fast. Back up everything before using this, it _may_ munch your files.

## Usage
Run this on a directory which contains `static_site.json`

This json file should look like the following (for the included example site):
```json
{
    "assets": [
        {
            "name": "header.html",
            "file": {
                "file": "static_assets/header.html"
            }
        }
    ],
    "files": [
        {
            "file": "index.html",
            "script": "index.js",
            "style": "index.css"
        },
        {
            "file": "nested_page/index.html",
            "style": "index.css"
        }
    ]
}
```

### Fields:
The file is parsed as a a `StaticSiteInfo`

`"assets"`: contains `StaticSiteAsset`s

- name: what you reference when using the `@@STATICIMPORT` preprocessor directive
- file: the corresponding `StaticSiteFile`
    - file: the file path
    - script (optional): the js script included with `@@STATICSCRIPTCOPY`
    - style (optional): the css style included with `@@STATICSTYLECOPY`

`"files"`: contains `StaticSiteFile`s


- file: the file path
- script (optional): the js script included with `@@STATICSCRIPTCOPY`
- style (optional): the css style included with `@@STATICSTYLECOPY`

### Directives:
To use a preprocessor directive, simply include it in a comment, like such:

`<!-- @@STATICSCRIPTCOPY -->` - copy the corresponding script file (see above)

`<!-- @@STATICSTYLECOPY -->` - copy the corresponding style file (see above)

`<!-- @@STATICIMPORT <some_asset> -->` - copy `<some_asset>` (see above)

The preprocessor will replace the comment with the text of the relevant file.
This _should_ work recursively too.

### Output:
The processed files should be output to `static_site_out` under the directory you ran it on.
