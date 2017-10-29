/*
Copyright (c) 2016 Redox OS Developers

MIT License

Link to original file: https://github.com/redox-os/kernel/blob/b023a715f9b1923da121360d643f6f343a5b5dc3/src/externs.rs
*/

use core::mem;

const WORD_SIZE: usize = mem::size_of::<usize>();

/// Memcpy
///
/// Copy N bytes of memory from one location to another.
///
/// This faster implementation works by copying bytes not one-by-one, but in
/// groups of 8 bytes (or 4 bytes in the case of 32-bit architectures).
#[no_mangle]
pub unsafe extern fn memcpy(dest: *mut u8, src: *const u8,
                            n: usize) -> *mut u8 {

    let n_usize: usize = n/WORD_SIZE; // Number of word sized groups
    let mut i: usize = 0;

    // Copy `WORD_SIZE` bytes at a time
    while i < n_usize {
        *((dest as usize + i) as *mut usize) =
            *((src as usize + i) as *const usize);
        i += WORD_SIZE;
    }

    // Copy 1 byte at a time
    while i < n {
        *((dest as usize + i) as *mut u8) = *((src as usize + i) as *const u8);
        i += 1;
    }

    dest
}

/// Memmove
///
/// Copy N bytes of memory from src to dest. The memory areas may overlap.
///
/// This faster implementation works by copying bytes not one-by-one, but in
/// groups of 8 bytes (or 4 bytes in the case of 32-bit architectures).
#[no_mangle]
pub unsafe extern fn memmove(dest: *mut u8, src: *const u8,
                             n: usize) -> *mut u8 {
    if src < dest as *const u8 {
        let n_usize: usize = n/WORD_SIZE; // Number of word sized groups
        let mut i: usize = n_usize*WORD_SIZE;

        // Copy `WORD_SIZE` bytes at a time
        while i != 0 {
            i -= WORD_SIZE;
            *((dest as usize + i) as *mut usize) =
                *((src as usize + i) as *const usize);
        }

        let mut i: usize = n;

        // Copy 1 byte at a time
        while i != n_usize*WORD_SIZE {
            i -= 1;
            *((dest as usize + i) as *mut u8) =
                *((src as usize + i) as *const u8);
        }
    } else {
        let n_usize: usize = n/WORD_SIZE; // Number of word sized groups
        let mut i: usize = 0;

        // Copy `WORD_SIZE` bytes at a time
        while i < n_usize {
            *((dest as usize + i) as *mut usize) =
                *((src as usize + i) as *const usize);
            i += WORD_SIZE;
        }

        // Copy 1 byte at a time
        while i < n {
            *((dest as usize + i) as *mut u8) =
                *((src as usize + i) as *const u8);
            i += 1;
        }
    }

    dest
}

/// Memset
///
/// Fill a block of memory with a specified value.
///
/// This faster implementation works by setting bytes not one-by-one, but in
/// groups of 8 bytes (or 4 bytes in the case of 32-bit architectures).
#[cfg(target_pointer_width = "64")]
#[no_mangle]
pub unsafe extern fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    let c = c as u64;
    let c = (c << 56) | (c << 48) | (c << 40) | (c << 32)
          | (c << 24) | (c << 16) | (c << 8)  | c;
    let n_64: usize = n/8;
    let mut i: usize = 0;

    // Set 8 bytes at a time
    while i < n_64 {
        *((dest as usize + i) as *mut u64) = c;
        i += 8;
    }

    let c = c as u8;

    // Set 1 byte at a time
    while i < n {
        *((dest as usize + i) as *mut u8) = c;
        i += 1;
    }

    dest
}

// 32-bit version of the function above
#[cfg(target_pointer_width = "32")]
#[no_mangle]
pub unsafe extern fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    let c = c as u32;
    let c = (c << 24) | (c << 16) | (c << 8)  | c;
    let n_32: usize = n/4;
    let mut i: usize = 0;

    // Set 4 bytes at a time
    while i < n_32 {
        *((dest as usize + i) as *mut u32) = c;
        i += 4;
    }

    let c = c as u8;

    // Set 1 byte at a time
    while i < n {
        *((dest as usize + i) as *mut u8) = c;
        i += 1;
    }

    dest
}

//function below doesn't work because of mem::transmute

/// Memset
///
/// Fill a block of memory with a specified value.
///
/// This faster implementation works by setting bytes not one-by-one, but in
/// groups of 8 bytes (or 4 bytes in the case of 32-bit architectures).
// #[no_mangle]
// pub unsafe extern fn memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
//     let c: usize = mem::transmute([c as u8; WORD_SIZE]);
//     let n_usize: usize = n/WORD_SIZE;
//     let mut i: usize = 0;

//     // Set `WORD_SIZE` bytes at a time
//     while i < n_usize {
//         *((dest as usize + i) as *mut usize) = c;
//         i += WORD_SIZE;
//     }

//     let c = c as u8;

//     // Set 1 byte at a time
//     while i < n {
//         *((dest as usize + i) as *mut u8) = c;
//         i += 1;
//     }

//     dest
// }

/// Memcmp
///
/// Compare two blocks of memory.
///
/// This faster implementation works by comparing bytes not one-by-one, but in
/// groups of 8 bytes (or 4 bytes in the case of 32-bit architectures).
#[no_mangle]
pub unsafe extern fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let n_usize: usize = n/WORD_SIZE;
    let mut i: usize = 0;

    while i < n_usize {
        let a = *((s1 as usize + i) as *const usize);
        let b = *((s2 as usize + i) as *const usize);
        if a != b {
            let n: usize = i + WORD_SIZE;
            // Find the one byte that is not equal
            while i < n {
                let a = *((s1 as usize + i) as *const u8);
                let b = *((s2 as usize + i) as *const u8);
                if a != b {
                    return a as i32 - b as i32;
                }
                i += 1;
            }
        }
        i += WORD_SIZE;
    }

    while i < n {
        let a = *((s1 as usize + i) as *const u8);
        let b = *((s2 as usize + i) as *const u8);
        if a != b {
            return a as i32 - b as i32;
        }
        i += 1;
    }

    0
}

