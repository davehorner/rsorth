ffi.load kernel32.dll as kernel32
ffi.fn kernel32 Sleep as Sleep ffi.u32 -> ffi.void

1000 Sleep
