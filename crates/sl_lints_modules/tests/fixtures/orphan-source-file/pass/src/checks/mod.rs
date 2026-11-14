//! Checks.

macro_rules! declare {
    ($($m:ident),*) => { $(mod $m;)* };
}

declare! { first, second }
