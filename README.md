# bed\_cov

This is a small utility for testing out different interval libraries.

## Implemented

- rust-lapper
- IITree aka cgranges aka ArrayBackedIntervalTree from rust-bio
- COITree (my impl is broken I think)

rust-lapper is about 3 seconds faster on the anno-rna set and 3 seconds slower on the rna-anno set. rust-lapper has a known worst case when there are intervals that span many smaller intervals.

## TODO

- Investigate the enclosedness of each of the two sets an how that affects rust lapper
- Add the indexed-lapper
