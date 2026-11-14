## Why

A guide is read from front to back. A page that takes longer than one sitting is left half read, and the reader returns to it without knowing where they stopped. Several shorter pages, each with one purpose, are finished.

Reading time is the page's words divided by `wpm`, plus time for each figure: the first one costs `image-first-seconds`, each one after it a little less, down to `image-min-seconds` after `image-falloff-count` figures. The result is an estimate. The message names the speed it assumed.

By default only guides are judged. A reference page is looked up, not read through, so its length is no burden.

## How to fix

Split the page where its purpose changes: a tutorial, then a page of concepts, then a reference. Move tables of options and long listings to a reference page and link them.
