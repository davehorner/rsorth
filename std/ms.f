( Sleep for N milliseconds using Rust FFI )

[include] std/ffi.f

ffi.load kernel32.dll as kernel32
ffi.fn kernel32 Sleep as Sleep ffi.u32 -> ffi.void

: ms ( n -- )
    ffi.u32 ffi.cast
    Sleep
;
