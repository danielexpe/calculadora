use std::ffi::CString;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use bytemuck::{Pod, Zeroable};
use thiserror::Error;

pub const FBIOGET_VSCREENINFO: libc::c_ulong = 0x4600;
pub const FBIOGET_FSCREENINFO: libc::c_ulong = 0x4602;
pub const FBIOPAN_DISPLAY: libc::c_ulong = 0x4606;
pub const KDGETMODE: libc::c_ulong = 0x4B3B;
pub const KDSETMODE: libc::c_ulong = 0x4B3A;
pub const KD_TEXT: libc::c_int = 0;
pub const KD_GRAPHICS: libc::c_int = 1;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Pod, Zeroable)]
pub struct FbBitfield {
    pub offset: u32,
    pub length: u32,
    pub msb_right: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct FbVarScreeninfo {
    pub xres: u32,
    pub yres: u32,
    pub xres_virtual: u32,
    pub yres_virtual: u32,
    pub xoffset: u32,
    pub yoffset: u32,
    pub bits_per_pixel: u32,
    pub grayscale: u32,
    pub red: FbBitfield,
    pub green: FbBitfield,
    pub blue: FbBitfield,
    pub transp: FbBitfield,
    pub nonstd: u32,
    pub activate: u32,
    pub height: u32,
    pub width: u32,
    pub accel_flags: u32,
    pub pixclock: u32,
    pub left_margin: u32,
    pub right_margin: u32,
    pub upper_margin: u32,
    pub lower_margin: u32,
    pub hsync_len: u32,
    pub vsync_len: u32,
    pub sync: u32,
    pub vmode: u32,
    pub rotate: u32,
    pub colorspace: u32,
    pub reserved: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
struct FbFixScreeninfo {
    id: [u8; 16],
    smem_start: libc::c_ulong,
    smem_len: u32,
    type_field: u32,
    type_aux: u32,
    visual: u32,
    xpanstep: u16,
    ypanstep: u16,
    ywrapstep: u16,
    line_length: u32,
    mmio_start: libc::c_ulong,
    mmio_len: u32,
    accel: u32,
    capabilities: u16,
    reserved: [u16; 2],
}

#[derive(Debug, Error)]
pub enum FbError {
    #[error("cannot open {path}: {source}")]
    Open { path: PathBuf, source: io::Error },
    #[error(
        "cannot open {path}: permission denied; add the user to the 'video' group \
         (sudo usermod -aG video $USER) and log in again"
    )]
    Permission { path: PathBuf, source: io::Error },
    #[error("ioctl {request} failed on {path}: {source}")]
    Ioctl {
        request: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    #[error(
        "unsupported framebuffer format: {bpp} bpp red {red}/{red_len} green {green}/{green_len} \
         blue {blue}/{blue_len}; expected 32/24 bpp 8-8-8 or 16 bpp 5-6-5"
    )]
    Unsupported {
        bpp: u32,
        red: u32,
        red_len: u32,
        green: u32,
        green_len: u32,
        blue: u32,
        blue_len: u32,
    },
    #[error("invalid framebuffer geometry: {0}")]
    Invalid(&'static str),
    #[error("cannot map framebuffer memory: {0}")]
    Map(io::Error),
    #[error("cannot open /dev/tty: {0}; run from a local virtual console (Ctrl+Alt+F3)")]
    ConsoleOpen(io::Error),
    #[error(
        "KDSETMODE failed: {0}; run from a local virtual console (Ctrl+Alt+F3), \
         not from X, Wayland or SSH"
    )]
    NotVt(io::Error),
    #[error("cannot restore console mode: {0}")]
    ConsoleMode(io::Error),
    #[error("frame buffer has {size} bytes; expected {expected}")]
    FrameSize { size: usize, expected: usize },
    #[error("cannot install signal handlers: {0}")]
    Signal(io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelFormat {
    pub bpp: u32,
    pub red: FbBitfield,
    pub green: FbBitfield,
    pub blue: FbBitfield,
    pub transp: FbBitfield,
}

impl PixelFormat {
    pub fn from_var(var: &FbVarScreeninfo) -> Result<Self, FbError> {
        let unsupported = || FbError::Unsupported {
            bpp: var.bits_per_pixel,
            red: var.red.offset,
            red_len: var.red.length,
            green: var.green.offset,
            green_len: var.green.length,
            blue: var.blue.offset,
            blue_len: var.blue.length,
        };

        let within = |field: FbBitfield, bits: u32| {
            field.offset <= bits && field.length <= bits - field.offset
        };
        let non_overlapping = |a: FbBitfield, b: FbBitfield| {
            let mask = |field: FbBitfield| {
                if field.length == 0 {
                    0
                } else {
                    (((1u64 << field.length) - 1) << field.offset) as u32
                }
            };
            mask(a) & mask(b) == 0
        };

        let format = match var.bits_per_pixel {
            32 | 24 => {
                let bits = var.bits_per_pixel;
                if var.red.length != 8
                    || var.green.length != 8
                    || var.blue.length != 8
                    || !within(var.red, bits)
                    || !within(var.green, bits)
                    || !within(var.blue, bits)
                    || !non_overlapping(var.red, var.green)
                    || !non_overlapping(var.red, var.blue)
                    || !non_overlapping(var.green, var.blue)
                    || !(if bits == 32 {
                        var.transp.length == 0
                            || (var.transp.length == 8
                                && within(var.transp, bits)
                                && non_overlapping(var.transp, var.red)
                                && non_overlapping(var.transp, var.green)
                                && non_overlapping(var.transp, var.blue))
                    } else {
                        var.transp.length == 0
                    })
                {
                    return Err(unsupported());
                }
                Self {
                    bpp: bits,
                    red: var.red,
                    green: var.green,
                    blue: var.blue,
                    transp: if bits == 32 {
                        var.transp
                    } else {
                        FbBitfield {
                            offset: 0,
                            length: 0,
                            msb_right: 0,
                        }
                    },
                }
            }
            16 => {
                if var.red.length != 5
                    || var.green.length != 6
                    || var.blue.length != 5
                    || !within(var.red, 16)
                    || !within(var.green, 16)
                    || !within(var.blue, 16)
                    || !non_overlapping(var.red, var.green)
                    || !non_overlapping(var.red, var.blue)
                    || !non_overlapping(var.green, var.blue)
                    || var.transp.length != 0
                {
                    return Err(unsupported());
                }
                Self {
                    bpp: 16,
                    red: var.red,
                    green: var.green,
                    blue: var.blue,
                    transp: FbBitfield {
                        offset: 0,
                        length: 0,
                        msb_right: 0,
                    },
                }
            }
            _ => return Err(unsupported()),
        };

        Ok(format)
    }

    pub const fn bytes_per_pixel(&self) -> usize {
        (self.bpp / 8) as usize
    }

    pub fn encode(&self, red: u8, green: u8, blue: u8) -> u32 {
        channel(red, self.red)
            | channel(green, self.green)
            | channel(blue, self.blue)
            | channel(255, self.transp)
    }
}

fn channel(value: u8, field: FbBitfield) -> u32 {
    if field.length == 0 {
        return 0;
    }
    let maximum = (1u32 << field.length) - 1;
    ((value as u32 * maximum + 127) / 255) << field.offset
}

pub fn write_pixels(
    format: &PixelFormat,
    line_length: u32,
    width: u32,
    height: u32,
    rgba: &[u8],
    output: &mut [u8],
) -> Result<(), FbError> {
    let expected = width as usize * height as usize * 4;
    if rgba.len() != expected {
        return Err(FbError::FrameSize {
            size: rgba.len(),
            expected,
        });
    }

    let stride = line_length as usize;
    let required = stride * height as usize;
    if output.len() < required {
        return Err(FbError::Invalid(
            "output buffer is smaller than line_length * height",
        ));
    }

    let bytes_per_pixel = format.bytes_per_pixel();
    for y in 0..height as usize {
        let row_start = y * stride;
        for x in 0..width as usize {
            let source = (y * width as usize + x) * 4;
            let word = format.encode(rgba[source], rgba[source + 1], rgba[source + 2]);
            let destination = row_start + x * bytes_per_pixel;
            for byte in 0..bytes_per_pixel {
                output[destination + byte] = ((word >> (byte * 8)) & 0xff) as u8;
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub struct FbInfo {
    pub width: u32,
    pub height: u32,
    pub line_length: u32,
    pub yres_virtual: u32,
    pub ypan_supported: bool,
    pub format: PixelFormat,
}

pub struct Framebuffer {
    _fd: OwnedFd,
    map: *mut u8,
    map_len: usize,
    info: FbInfo,
}

impl Framebuffer {
    pub fn open(path: &Path) -> Result<Self, FbError> {
        let path = path.to_path_buf();
        let c_path = CString::new(path.as_os_str().as_bytes()).map_err(|_| FbError::Open {
            path: path.clone(),
            source: io::Error::new(io::ErrorKind::InvalidInput, "path contains a NUL byte"),
        })?;

        // SAFETY: c_path is a valid NUL-terminated path and open does not retain the pointer.
        let raw_fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDWR | libc::O_CLOEXEC) };
        if raw_fd < 0 {
            let source = io::Error::last_os_error();
            return Err(if source.raw_os_error() == Some(libc::EACCES) {
                FbError::Permission { path, source }
            } else {
                FbError::Open { path, source }
            });
        }
        // SAFETY: raw_fd is a newly opened descriptor owned exclusively by this value.
        let fd = unsafe { OwnedFd::from_raw_fd(raw_fd) };

        let mut var = FbVarScreeninfo::zeroed();
        ioctl_mut(fd.as_raw_fd(), FBIOGET_VSCREENINFO, &mut var).map_err(|source| {
            FbError::Ioctl {
                request: "FBIOGET_VSCREENINFO",
                path: path.clone(),
                source,
            }
        })?;

        // SAFETY: the kernel writes a complete fb_fix_screeninfo, and the struct is repr(C).
        let mut fix: FbFixScreeninfo = unsafe { std::mem::zeroed() };
        ioctl_mut(fd.as_raw_fd(), FBIOGET_FSCREENINFO, &mut fix).map_err(|source| {
            FbError::Ioctl {
                request: "FBIOGET_FSCREENINFO",
                path: path.clone(),
                source,
            }
        })?;

        if var.xres == 0 || var.yres == 0 {
            return Err(FbError::Invalid("xres and yres must be greater than zero"));
        }
        if var.xres_virtual < var.xres || var.yres_virtual < var.yres {
            return Err(FbError::Invalid(
                "virtual resolution must cover the visible resolution",
            ));
        }

        let format = PixelFormat::from_var(&var)?;
        let minimum_stride = var.xres * format.bytes_per_pixel() as u32;
        if fix.line_length < minimum_stride {
            return Err(FbError::Invalid(
                "line_length is smaller than xres * bytes_per_pixel",
            ));
        }
        if fix.smem_len < fix.line_length * var.yres {
            return Err(FbError::Invalid(
                "smem_len is smaller than the visible frame",
            ));
        }

        let map_len = fix.line_length as usize * var.yres as usize;
        // SAFETY: map_len covers the visible frame, fd is valid and MAP_SHARED is required by fbdev.
        let map = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                map_len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd.as_raw_fd(),
                0,
            )
        };
        if map == libc::MAP_FAILED {
            return Err(FbError::Map(io::Error::last_os_error()));
        }

        let ypan_supported = fix.ypanstep > 0
            && var.yres_virtual >= 2 * var.yres
            && var.yres % u32::from(fix.ypanstep) == 0;

        Ok(Self {
            _fd: fd,
            map: map.cast(),
            map_len,
            info: FbInfo {
                width: var.xres,
                height: var.yres,
                line_length: fix.line_length,
                yres_virtual: var.yres_virtual,
                ypan_supported,
                format,
            },
        })
    }

    pub fn info(&self) -> &FbInfo {
        &self.info
    }

    pub fn present(&mut self, rgba: &[u8]) -> Result<(), FbError> {
        // SAFETY: the mapping is live for self, spans map_len bytes and is exclusively borrowed.
        let output = unsafe { std::slice::from_raw_parts_mut(self.map, self.map_len) };
        write_pixels(
            &self.info.format,
            self.info.line_length,
            self.info.width,
            self.info.height,
            rgba,
            output,
        )
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        // SAFETY: map was returned by mmap with map_len bytes and is unmapped exactly once.
        unsafe {
            libc::munmap(self.map.cast(), self.map_len);
        }
    }
}

fn ioctl_mut<T>(fd: libc::c_int, request: libc::c_ulong, value: &mut T) -> io::Result<()> {
    // SAFETY: the request expects a writable pointer to the matching kernel struct.
    let result = unsafe { libc::ioctl(fd, request, value as *mut T) };
    if result < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

static CONSOLE_FD: AtomicI32 = AtomicI32::new(-1);
static CONSOLE_PREVIOUS: AtomicI32 = AtomicI32::new(KD_TEXT);
static QUIT_REQUESTED: AtomicBool = AtomicBool::new(false);

pub struct ConsoleGuard {
    fd: OwnedFd,
}

impl ConsoleGuard {
    pub fn enter() -> Result<Self, FbError> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .map_err(FbError::ConsoleOpen)?;
        let fd = OwnedFd::from(file);
        let raw = fd.as_raw_fd();

        let mut previous: libc::c_int = KD_TEXT;
        // SAFETY: KDGETMODE writes an int through the provided valid pointer.
        if unsafe { libc::ioctl(raw, KDGETMODE, &mut previous) } < 0 {
            return Err(FbError::ConsoleMode(io::Error::last_os_error()));
        }

        // SAFETY: KDSETMODE takes the mode by value and only mutates kernel state for this tty.
        if unsafe { libc::ioctl(raw, KDSETMODE, KD_GRAPHICS) } < 0 {
            let source = io::Error::last_os_error();
            return Err(if source.raw_os_error() == Some(libc::ENOTTY) {
                FbError::NotVt(source)
            } else {
                FbError::ConsoleMode(source)
            });
        }

        let guard = Self { fd };
        CONSOLE_PREVIOUS.store(previous, Ordering::SeqCst);
        CONSOLE_FD.store(guard.fd.as_raw_fd(), Ordering::SeqCst);
        write_all(guard.fd.as_raw_fd(), b"\x1b[?25l\x1b[2J");
        Ok(guard)
    }
}

impl Drop for ConsoleGuard {
    fn drop(&mut self) {
        let fd = self.fd.as_raw_fd();
        // SAFETY: restoring the saved mode is safe for the tty owned by this guard.
        unsafe {
            libc::ioctl(fd, KDSETMODE, CONSOLE_PREVIOUS.load(Ordering::SeqCst));
        }
        write_all(fd, b"\x1b[?25h\x1b[2J");
        CONSOLE_FD.store(-1, Ordering::SeqCst);
    }
}

pub fn restore_console_best_effort() {
    let fd = CONSOLE_FD.load(Ordering::SeqCst);
    if fd >= 0 {
        // SAFETY: fd refers to the console tty opened by ConsoleGuard.
        unsafe {
            libc::ioctl(fd, KDSETMODE, CONSOLE_PREVIOUS.load(Ordering::SeqCst));
        }
        write_all(fd, b"\x1b[?25h\x1b[2J");
    }
    let _ = crossterm::terminal::disable_raw_mode();
}

fn write_all(fd: libc::c_int, mut bytes: &[u8]) {
    while !bytes.is_empty() {
        // SAFETY: bytes is a valid readable buffer for its length and fd is a valid tty.
        let written = unsafe { libc::write(fd, bytes.as_ptr().cast(), bytes.len()) };
        if written <= 0 {
            break;
        }
        bytes = &bytes[written as usize..];
    }
}

pub fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_console_best_effort();
        default(info);
    }));
}

extern "C" fn handle_signal(_signal: libc::c_int) {
    QUIT_REQUESTED.store(true, Ordering::SeqCst);
}

pub fn install_signal_handlers() -> Result<(), FbError> {
    for signal in [libc::SIGINT, libc::SIGTERM, libc::SIGHUP] {
        // SAFETY: zeroed sigaction is valid; the handler only touches an atomic.
        unsafe {
            let mut action: libc::sigaction = std::mem::zeroed();
            action.sa_sigaction = handle_signal as *const () as usize;
            action.sa_flags = libc::SA_RESTART;
            libc::sigemptyset(&mut action.sa_mask);
            if libc::sigaction(signal, &action, std::ptr::null_mut()) < 0 {
                return Err(FbError::Signal(io::Error::last_os_error()));
            }
        }
    }
    Ok(())
}

pub fn quit_requested() -> bool {
    QUIT_REQUESTED.load(Ordering::SeqCst)
}

const _: () = assert!(std::mem::size_of::<FbVarScreeninfo>() == 160);

#[cfg(test)]
mod tests {
    use super::*;

    fn bitfield(offset: u32, length: u32) -> FbBitfield {
        FbBitfield {
            offset,
            length,
            msb_right: 0,
        }
    }

    fn var(bpp: u32, red: FbBitfield, green: FbBitfield, blue: FbBitfield) -> FbVarScreeninfo {
        let mut var = FbVarScreeninfo::zeroed();
        var.xres = 1920;
        var.yres = 1080;
        var.xres_virtual = 1920;
        var.yres_virtual = 1080;
        var.bits_per_pixel = bpp;
        var.red = red;
        var.green = green;
        var.blue = blue;
        var
    }

    #[test]
    fn abi_sizes_match_linux_x86_64() {
        assert_eq!(std::mem::size_of::<FbVarScreeninfo>(), 160);
        assert_eq!(std::mem::size_of::<FbFixScreeninfo>(), 80);
    }

    #[test]
    fn parses_32bpp_bgrx() {
        let format =
            PixelFormat::from_var(&var(32, bitfield(16, 8), bitfield(8, 8), bitfield(0, 8)))
                .unwrap();
        assert_eq!(format.bytes_per_pixel(), 4);
        assert_eq!(format.encode(0x12, 0x34, 0x56), 0x0012_3456);
    }

    #[test]
    fn fills_alpha_bits_as_opaque() {
        let mut var = var(32, bitfield(16, 8), bitfield(8, 8), bitfield(0, 8));
        var.transp = bitfield(24, 8);
        let format = PixelFormat::from_var(&var).unwrap();
        assert_eq!(format.encode(0x12, 0x34, 0x56), 0xff12_3456);
    }

    #[test]
    fn parses_24bpp_rgb() {
        let format =
            PixelFormat::from_var(&var(24, bitfield(16, 8), bitfield(8, 8), bitfield(0, 8)))
                .unwrap();
        assert_eq!(format.bytes_per_pixel(), 3);
        assert_eq!(format.encode(0x12, 0x34, 0x56), 0x0012_3456);
    }

    #[test]
    fn parses_16bpp_rgb565() {
        let format =
            PixelFormat::from_var(&var(16, bitfield(11, 5), bitfield(5, 6), bitfield(0, 5)))
                .unwrap();
        assert_eq!(format.bytes_per_pixel(), 2);
        assert_eq!(format.encode(255, 255, 255), 0xffff);
        assert_eq!(format.encode(0, 0, 0), 0);
        assert_eq!(format.encode(255, 0, 0), 0xf800);
        assert_eq!(format.encode(0, 255, 0), 0x07e0);
        assert_eq!(format.encode(0, 0, 255), 0x001f);
    }

    #[test]
    fn rejects_unknown_bpp_and_bad_channels() {
        assert!(matches!(
            PixelFormat::from_var(&var(8, bitfield(0, 8), bitfield(0, 8), bitfield(0, 8))),
            Err(FbError::Unsupported { .. })
        ));
        assert!(matches!(
            PixelFormat::from_var(&var(32, bitfield(0, 8), bitfield(0, 8), bitfield(0, 8))),
            Err(FbError::Unsupported { .. })
        ));
        assert!(matches!(
            PixelFormat::from_var(&var(16, bitfield(11, 5), bitfield(5, 5), bitfield(0, 5))),
            Err(FbError::Unsupported { .. })
        ));
        assert!(matches!(
            PixelFormat::from_var(&var(32, bitfield(16, 8), bitfield(8, 7), bitfield(0, 8))),
            Err(FbError::Unsupported { .. })
        ));
    }

    #[test]
    fn writes_rows_with_stride_padding() {
        let format =
            PixelFormat::from_var(&var(32, bitfield(16, 8), bitfield(8, 8), bitfield(0, 8)))
                .unwrap();
        let rgba = [
            255u8, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ];
        let mut output = vec![0xAA; 24];
        write_pixels(&format, 12, 2, 2, &rgba, &mut output).unwrap();
        assert_eq!(&output[0..4], &[0, 0, 255, 0]);
        assert_eq!(&output[4..8], &[0, 255, 0, 0]);
        assert_eq!(&output[8..12], &[0xAA; 4]);
        assert_eq!(&output[12..16], &[255, 0, 0, 0]);
        assert_eq!(&output[16..20], &[255, 255, 255, 0]);
        assert_eq!(&output[20..24], &[0xAA; 4]);
    }

    #[test]
    fn rejects_wrong_frame_size() {
        let format =
            PixelFormat::from_var(&var(32, bitfield(16, 8), bitfield(8, 8), bitfield(0, 8)))
                .unwrap();
        let mut output = vec![0; 8];
        assert!(matches!(
            write_pixels(&format, 8, 2, 1, &[0; 4], &mut output),
            Err(FbError::FrameSize { .. })
        ));
        assert!(matches!(
            write_pixels(&format, 8, 2, 1, &[0; 8], &mut [0; 4]),
            Err(FbError::Invalid(_))
        ));
    }

    #[test]
    fn channel_scales_by_bitfield_length() {
        assert_eq!(channel(255, bitfield(0, 8)), 255);
        assert_eq!(channel(255, bitfield(11, 5)), 0xf800);
        assert_eq!(channel(0, bitfield(0, 8)), 0);
        assert_eq!(channel(255, bitfield(0, 0)), 0);
    }

    #[test]
    fn backend_errors_are_actionable() {
        let permission = FbError::Permission {
            path: PathBuf::from("/dev/fb0"),
            source: io::Error::from_raw_os_error(libc::EACCES),
        };
        assert!(permission.to_string().contains("video"));
        assert!(permission.to_string().contains("/dev/fb0"));

        let not_vt = FbError::NotVt(io::Error::from_raw_os_error(libc::ENOTTY));
        assert!(not_vt.to_string().contains("virtual console"));

        let unsupported = FbError::Unsupported {
            bpp: 8,
            red: 0,
            red_len: 8,
            green: 0,
            green_len: 8,
            blue: 0,
            blue_len: 8,
        };
        assert!(unsupported.to_string().contains("expected"));
    }

    #[test]
    fn signal_flag_is_atomic_and_sticky() {
        QUIT_REQUESTED.store(false, Ordering::SeqCst);
        assert!(!quit_requested());
        handle_signal(libc::SIGTERM);
        assert!(quit_requested());
        QUIT_REQUESTED.store(false, Ordering::SeqCst);
    }
}
