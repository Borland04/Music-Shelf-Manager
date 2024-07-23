# Music manager
This application manages downloaded audio by moving them into the folders with predefined hierarchy.

### Directory structure
`<Root directory>/<Album artist>/<Album>/<Title>.mp3(or other extension)`.

*Artist/Album/Title information is taken from __ID3v1 | ID3v2__*

## Notes

**IMPORTANT!** It is for personal use - app did not suppose to be flexible - just hardcoded folder structure.

**NOTE!** This is my early try of Rustlang. Be caution, source code might cause an eye-bleeding

## TODOs

* Destination directory pattern - instead of hardcoded `<ROOT>/<Artist>/<Album>/<Title>.mp3`
* Support other formats
* Additional warnings (such as: "No Lyrics", "No cover", ...)
* Automatically search for missing lyrics?
