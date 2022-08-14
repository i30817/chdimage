**Chd parser Python binding**
=============================

This small project is a binding to the chd part of the rust project `imageparse <https://github.com/Manorhos/imageparse>`_ done for its capability to access chd files decompressed bytes, even those with parents, recognize the track boundaries, and calculate their sha1 checksums.

Upstream has some limitations still:

1. Will not load hard drive chds, since it's out of scope of the project.
2. Will not yet read gdi chds, although that's possible in the future.
3. Like the upstream it depends on, `chd-rs <https://github.com/SnowflakePowered/chd-rs/issues>`_ it can't write a new chd.
4. It is not actually a filesystem mounter, it only accesses the raw decompressed bytes and recognizes track boundaries.

You can access the chd track sha1 checksums of `b.chd` with parent `a.chd` like this::

    import chdimage
    chd = chdimage.open_with_parent('b.chd', ['a.chd'])
    sha1sums = [ bytes(x).hex() for x in chd.track_sha1s() ]

Chd files that aren't parents are ignored, so you can choose your own strategy to find parents::

    chd = chdimage.open_with_parent('b.chd', ['not_b_parent.chd', 'a.chd'])

You can iterate over the track bytes like this::

   for x in range(0,chd.num_tracks()):
     event = None
     #tracks go from 1 to num_tracks(), 0 does not count
     print(f'track {x+1}')
     while event != Event.TRACKCHANGE and event != Event.ENDOFDISC:
       sector = chd.copy_current_sector()
       event = chd.advance_position()
       #printing bytes is useless and lossy, do something else
       print(bytes(sector).decode(errors='replace'))
