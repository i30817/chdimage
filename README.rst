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
    
Likewise, if a file does not require a parent, the second argument list is ignored even if you use it::

    chd = chdimage.open_with_parent('a.chd', ['not_b_parent.chd', 'c.chd'])

If you're absolutely sure that the file you have has no parent you can use the open function only, although you'd probably need the python bindings to chd-rs, `chd-rs-py <https://github.com/chyyran/chd-rs-py>`_ to figure that out, so you probably want to use `open_with_parent` always::

    from chd import chd_read_header
    header = chd_read_header("a.chd")
    if not header.has_parent():
      chd = chdimage.open('a.chd')

You can iterate over the track bytes like this::

   for x in range(0,chd.num_tracks()):
     event = None
     #tracks go from 1 to num_tracks(), the functions that
     #take track numbers require x+1 if you use this range
     print(f'track {x+1}')
     while event != Event.TRACKCHANGE and event != Event.ENDOFDISC:
       sector = chd.copy_current_sector()
       event = chd.advance_position()
       #printing bytes is useless and lossy, do something else
       print(bytes(sector).decode(errors='replace'))

Find the first binary track::

   for x in range(1,chd.num_tracks()+1):
     chd.set_location_to_track(x)
     if chd.current_track_type() == TrackType.MODE1
       or chd.current_track_type() == TrackType.MODE2:
       print(f'track {x}')

Get get a sector at a location based on lba (logical block addressing) or msf (minutes, seconds, frame)::

   lba = MsfIndex.from_lba(100)
   msf = MsfIndex(0, 1, 25)
   assert(lba == msf)
   chd.set_location(lba)
   sector_lba = chd.copy_current_sector()
   chd.set_location(msf)
   sector_msf = chd.copy_current_sector()
   assert(sector_lba == sector_msf)
