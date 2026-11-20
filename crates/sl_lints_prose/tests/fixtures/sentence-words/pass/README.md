# fx

Short sentences, e.g. this one, pass. Version 3.5 is out.

A writer that finds the file held by another process gets [`FsError::Locked`]. The lock stays with the first process until that process closes the file, drops its handle or exits. A reader that asks for the length gets `None`. It asks again later, after the writer has closed the file.

| A table row with very many words in it is not prose that the sentence rule checks at all, ever |
