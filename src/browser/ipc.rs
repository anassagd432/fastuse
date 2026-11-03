//! Local connection between the MCP server and the native host the browser starts:
//! a named pipe on Windows (only this user and SYSTEM may open it), a Unix socket inside a private
//! folder elsewhere. Both are byte streams used as strict request / response lines, so a stream
//! is never read and written at the same time (that would deadlock a synchronous Windows pipe).

use std::io;

#[cfg(windows)]
pub use win::{connect, Listener, Stream};
#[cfg(unix)]
pub use unix::{connect, Listener, Stream};

/// Where the host of process `pid` listens.
pub fn endpoint_name(pid: u32) -> String {
    #[cfg(windows)]
    {
        let user = std::env::var("USERNAME").unwrap_or_default();
        format!(r"\\.\pipe\fastuse-browser-{}-{pid}", user.replace(['\\', '/', ' '], "_"))
    }
    #[cfg(unix)]
    {
        super::data_dir().join("sock").join(format!("{pid}.sock")).display().to_string()
    }
}

#[allow(dead_code)]
fn unsupported() -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, "no local IPC on this platform")
}

#[cfg(windows)]
mod win {
    use std::fs::{File, OpenOptions};
    use std::io;
    use std::os::windows::io::{FromRawHandle, RawHandle};
    use std::sync::Mutex;

    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::{GetLastError, ERROR_PIPE_CONNECTED, HANDLE, HLOCAL, LocalFree};
    use windows::Win32::Security::Authorization::{ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1};
    use windows::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};
    use windows::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
    use windows::Win32::System::Pipes::{
        ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE,
        PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
    };

    pub type Stream = File;

    pub struct Listener {
        name: Vec<u16>,
        sd: isize,
        /// the next free pipe instance (created ahead so clients never find the pipe missing)
        pending: Mutex<isize>,
    }

    unsafe fn make_instance(name: &[u16], sd: isize) -> io::Result<isize> {
        let sa = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: sd as *mut std::ffi::c_void,
            bInheritHandle: false.into(),
        };
        let h = CreateNamedPipeW(
            PCWSTR(name.as_ptr()),
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
            PIPE_UNLIMITED_INSTANCES,
            1 << 16,
            1 << 16,
            0,
            Some(&sa),
        );
        if h.is_invalid() {
            return Err(io::Error::last_os_error());
        }
        Ok(h.0 as isize)
    }

    impl Listener {
        pub fn bind(name: &str) -> io::Result<Listener> {
            let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
            let mut psd = PSECURITY_DESCRIPTOR::default();
            // protected DACL: full access for the owner (this user) and for SYSTEM, nobody else
            unsafe {
                ConvertStringSecurityDescriptorToSecurityDescriptorW(w!("D:P(A;;GA;;;OW)(A;;GA;;;SY)"), SDDL_REVISION_1, &mut psd, None)
                    .map_err(|e| io::Error::other(e.to_string()))?;
            }
            let sd = psd.0 as isize;
            let first = unsafe { make_instance(&name, sd)? };
            Ok(Listener { name, sd, pending: Mutex::new(first) })
        }

        pub fn accept(&self) -> io::Result<Stream> {
            let h = *self.pending.lock().unwrap();
            unsafe {
                let handle = HANDLE(h as *mut std::ffi::c_void);
                if ConnectNamedPipe(handle, None).is_err() && GetLastError() != ERROR_PIPE_CONNECTED {
                    return Err(io::Error::last_os_error());
                }
                *self.pending.lock().unwrap() = make_instance(&self.name, self.sd)?;
                Ok(File::from_raw_handle(h as RawHandle))
            }
        }
    }

    impl Drop for Listener {
        fn drop(&mut self) {
            unsafe {
                let _ = LocalFree(Some(HLOCAL(self.sd as *mut std::ffi::c_void)));
            }
        }
    }

    pub fn connect(name: &str) -> io::Result<Stream> {
        OpenOptions::new().read(true).write(true).open(name)
    }
}

#[cfg(unix)]
mod unix {
    use std::io;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::Path;

    pub type Stream = UnixStream;

    pub struct Listener(UnixListener, String);

    impl Listener {
        pub fn bind(name: &str) -> io::Result<Listener> {
            let p = Path::new(name);
            if let Some(dir) = p.parent() {
                std::fs::create_dir_all(dir)?;
                std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
            }
            let _ = std::fs::remove_file(p);
            let l = UnixListener::bind(p)?;
            std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o600))?;
            Ok(Listener(l, name.to_string()))
        }

        pub fn accept(&self) -> io::Result<Stream> {
            self.0.accept().map(|(s, _)| s)
        }
    }

    impl Drop for Listener {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.1);
        }
    }

    pub fn connect(name: &str) -> io::Result<Stream> {
        UnixStream::connect(name)
    }
}
