use addr2line::Context;

// PERSONAL NOTES/TODOs:
//
//      For only unwinding, `.eh_frame` and `.eh_frame_hdr` are enough.
//
//      For debug builds, either manually split debug information in a custom
//      kernel object (/ module ? namely `debug-info.zko`), or ask cargo to
//      do it with `split-debuginfo=packed` and `debuginfo=2` or more
