# contributing

ay man thank you for trying to contribute to my repo  
before opening a pr, PLEASE keep the same coding style as me

> tldr: fully qualified paths, 4-space indent, ~100 char lines, one thing per line, no abbreviations, ? over match, minimal imports, and inline values instead of assigning then returning (just don’t include a semicolon).

## style: 

cbg’s code style is pretty simple (I believe):

- fully qualified paths for one-offs (`std::fs::read_to_string`, not `read_to_string`)

### exceptions:  
use poise::serenity_prelude as serenity; and in cbg-core (you SHOULD still use fully qualified paths just to make life easier you should just `use` anyways lol) 

## imports: 

- keep em' minimal (see style): 

- group all imoorts from a crate in ONE use statement (in the future I'll make rustfmt.toml)

- if something’s only used inside a function, `use` it inside the function (if used in 2 functions it's ok) 


- indent: 4 spaces (I might make editorconfig) 

- line width: around 100 chars

- lowercase comments and output (ehh i don't care about comments but output is a half must) 

- inline small values instead of assigning them:
```rust
let result = x + y; result // bad
x + y // good
```

- variable names: no abbreviations (mostly) 

use user, not u

use stats, not s

### exceptions: img or id, (mostly) super-common abbreviations

- no single-letter names, even in closures


- one obvious thing per line, avoid chaining if it hides intent

- prefer ? over match for simple error propagation

- comment tone: short, lowercase?, and to the point

- error handling:

cbg-core uses thiserror (mostly) 

cbg-bot uses NOTHING!! maybe soon I'll make anyhow



## formatting: 

you can run this before pushing:

```bash
cargo fmt
cargo clippy --fix
cargo check
```

> side note: if you see broken style by me, that wasn't me I swear I wasn't coding until 2 weeks ago
>
> in all seriousness tho you would be prime m2k if you fixed those frfr