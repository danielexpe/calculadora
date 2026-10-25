use std::ffi::{c_char, c_int, c_uint, c_ulong, c_void, CString};
use std::ptr;
use std::sync::mpsc::{Receiver, Sender};

use libloading::{Library, Symbol};

use crate::audio::{pcm16, SoundEvent, SAMPLE_RATE};

type PcmHandle = *mut c_void;
type OpenFn = unsafe extern "C" fn(*mut PcmHandle, *const c_char, c_int, c_int) -> c_int;
type SetParamsFn =
    unsafe extern "C" fn(PcmHandle, c_int, c_int, c_uint, c_uint, c_int, c_uint) -> c_int;
type WriteFn = unsafe extern "C" fn(PcmHandle, *const c_void, c_ulong) -> libc::c_long;
type RecoverFn = unsafe extern "C" fn(PcmHandle, c_int, c_int) -> c_int;
type CloseFn = unsafe extern "C" fn(PcmHandle) -> c_int;

const SND_PCM_STREAM_PLAYBACK: c_int = 0;
const SND_PCM_FORMAT_S16_LE: c_int = 2;
const SND_PCM_ACCESS_RW_INTERLEAVED: c_int = 3;
const LATENCY_US: c_uint = 100_000;

pub fn run(receiver: Receiver<SoundEvent>, ready: Sender<bool>) {
    let Ok(mut backend) = AlsaBackend::open() else {
        let _ = ready.send(false);
        return;
    };

    if ready.send(true).is_err() {
        return;
    }

    while let Ok(event) = receiver.recv() {
        backend.play(&pcm16(event));
    }
}

struct AlsaBackend {
    _library: Library,
    handle: PcmHandle,
    write: WriteFn,
    recover: RecoverFn,
    close: CloseFn,
}

impl AlsaBackend {
    fn open() -> Result<Self, Box<dyn std::error::Error>> {
        // SAFETY: the library stays alive in self and every symbol is copied before use.
        unsafe {
            let library = Library::new("libasound.so.2")?;
            let open: Symbol<OpenFn> = library.get(b"snd_pcm_open\0")?;
            let set_params: Symbol<SetParamsFn> = library.get(b"snd_pcm_set_params\0")?;
            let write: Symbol<WriteFn> = library.get(b"snd_pcm_writei\0")?;
            let recover: Symbol<RecoverFn> = library.get(b"snd_pcm_recover\0")?;
            let close: Symbol<CloseFn> = library.get(b"snd_pcm_close\0")?;

            let device = CString::new("default")?;
            let mut handle: PcmHandle = ptr::null_mut();
            let result = open(&mut handle, device.as_ptr(), SND_PCM_STREAM_PLAYBACK, 0);
            if result < 0 {
                return Err(format!("ALSA snd_pcm_open failed: {result}").into());
            }

            let result = set_params(
                handle,
                SND_PCM_FORMAT_S16_LE,
                SND_PCM_ACCESS_RW_INTERLEAVED,
                1,
                SAMPLE_RATE,
                1,
                LATENCY_US,
            );
            if result < 0 {
                close(handle);
                return Err(format!("ALSA snd_pcm_set_params failed: {result}").into());
            }

            let write_fn = *write;
            let recover_fn = *recover;
            let close_fn = *close;
            Ok(Self {
                _library: library,
                handle,
                write: write_fn,
                recover: recover_fn,
                close: close_fn,
            })
        }
    }

    fn play(&mut self, samples: &[i16]) {
        let bytes = bytemuck::cast_slice::<i16, u8>(samples);
        let mut offset = 0usize;

        while offset < samples.len() {
            let frame_offset = offset * 2;
            let remaining = samples.len() - offset;
            // SAFETY: the buffer pointer is valid for the remaining frames and ALSA copies it.
            let written = unsafe {
                (self.write)(
                    self.handle,
                    bytes[frame_offset..].as_ptr().cast(),
                    remaining as c_ulong,
                )
            };

            if written < 0 {
                // SAFETY: recovery only touches the PCM handle owned by this backend.
                let recovered = unsafe { (self.recover)(self.handle, written as c_int, 1) };
                if recovered < 0 {
                    break;
                }
            } else if written == 0 {
                break;
            } else {
                offset += written as usize;
            }
        }
    }
}

impl Drop for AlsaBackend {
    fn drop(&mut self) {
        // SAFETY: close is called exactly once for the handle opened by this backend.
        unsafe {
            (self.close)(self.handle);
        }
    }
}
