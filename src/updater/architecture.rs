#[cfg(target_arch = "x86_64")]
use windows::core::{BOOL, PCSTR, PCWSTR};
#[cfg(target_arch = "x86_64")]
use windows::Win32::Foundation::{GetLastError, HANDLE};
#[cfg(target_arch = "x86_64")]
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
#[cfg(target_arch = "x86_64")]
use windows::Win32::System::Threading::GetCurrentProcess;

const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
const IMAGE_FILE_MACHINE_ARM64: u16 = 0xaa64;
#[cfg(target_arch = "x86_64")]
const IMAGE_FILE_MACHINE_UNKNOWN: u16 = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Architecture {
    // Both asset architectures are needed by the shared PE verifier and tests.
    #[allow(dead_code)]
    X64,
    Arm64,
}

impl Architecture {
    pub(super) const fn asset_name(self) -> &'static str {
        match self {
            Self::X64 => "claude-code-usage-monitor.exe",
            Self::Arm64 => "claude-code-usage-monitor-arm64.exe",
        }
    }

    pub(super) const fn machine(self) -> u16 {
        match self {
            Self::X64 => IMAGE_FILE_MACHINE_AMD64,
            Self::Arm64 => IMAGE_FILE_MACHINE_ARM64,
        }
    }

    pub(super) fn for_update() -> Self {
        #[cfg(target_arch = "aarch64")]
        {
            Self::Arm64
        }

        #[cfg(target_arch = "x86_64")]
        {
            detected_or_x64(native_architecture())
        }
    }
}

#[cfg(target_arch = "x86_64")]
fn detected_or_x64(detected: Result<Architecture, String>) -> Architecture {
    match detected {
        Ok(architecture) => architecture,
        Err(error) => {
            crate::diagnose::log(format!(
                "unable to detect native Windows architecture; using x64 update: {error}"
            ));
            Architecture::X64
        }
    }
}

#[cfg(target_arch = "x86_64")]
fn native_architecture() -> Result<Architecture, String> {
    // Resolve this API at runtime so the x64 build still starts on older Windows
    // versions where IsWow64Process2 is unavailable.
    type IsWow64Process2Fn = unsafe extern "system" fn(HANDLE, *mut u16, *mut u16) -> BOOL;

    let kernel32_name: Vec<u16> = "kernel32.dll\0".encode_utf16().collect();
    let kernel32 = unsafe { GetModuleHandleW(PCWSTR::from_raw(kernel32_name.as_ptr())) }
        .map_err(|error| format!("unable to locate kernel32.dll: {error}"))?;
    let function = unsafe { GetProcAddress(kernel32, PCSTR(b"IsWow64Process2\0".as_ptr())) }
        .ok_or_else(|| "IsWow64Process2 is unavailable".to_string())?;
    let function: IsWow64Process2Fn = unsafe { std::mem::transmute(function) };

    let mut process_machine = 0u16;
    let mut native_machine = 0u16;
    let success = unsafe {
        function(
            GetCurrentProcess(),
            &mut process_machine,
            &mut native_machine,
        )
    };
    if !success.as_bool() {
        let error = unsafe { GetLastError() };
        return Err(format!(
            "IsWow64Process2 failed with Windows error {}",
            error.0
        ));
    }

    // The native machine identifies the host for both native x64 processes and
    // x64 processes running under ARM64 emulation.
    architecture_for_machines(process_machine, native_machine).ok_or_else(|| {
        format!(
            "unsupported Windows process/native machine types 0x{process_machine:04x}/0x{native_machine:04x}"
        )
    })
}

#[cfg(target_arch = "x86_64")]
fn architecture_for_machines(process_machine: u16, native_machine: u16) -> Option<Architecture> {
    if process_machine != IMAGE_FILE_MACHINE_UNKNOWN && process_machine != IMAGE_FILE_MACHINE_AMD64
    {
        return None;
    }
    match native_machine {
        IMAGE_FILE_MACHINE_AMD64 => Some(Architecture::X64),
        IMAGE_FILE_MACHINE_ARM64 => Some(Architecture::Arm64),
        _ => None,
    }
}

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;

    #[test]
    fn maps_supported_native_machine_types() {
        assert_eq!(
            architecture_for_machines(IMAGE_FILE_MACHINE_UNKNOWN, IMAGE_FILE_MACHINE_AMD64),
            Some(Architecture::X64)
        );
        assert_eq!(
            architecture_for_machines(IMAGE_FILE_MACHINE_AMD64, IMAGE_FILE_MACHINE_ARM64),
            Some(Architecture::Arm64)
        );
    }

    #[test]
    fn rejects_unknown_native_machine_types() {
        assert_eq!(
            architecture_for_machines(IMAGE_FILE_MACHINE_UNKNOWN, 0),
            None
        );
        assert_eq!(
            architecture_for_machines(0x01c4, IMAGE_FILE_MACHINE_ARM64),
            None
        );
    }

    #[test]
    fn uses_x64_when_native_architecture_detection_fails() {
        assert_eq!(
            detected_or_x64(Err("simulated failure".into())),
            Architecture::X64
        );
        assert_eq!(
            detected_or_x64(Ok(Architecture::Arm64)),
            Architecture::Arm64
        );
    }

    #[test]
    fn detects_the_native_architecture_of_the_current_process() {
        assert!(matches!(
            native_architecture().unwrap(),
            Architecture::X64 | Architecture::Arm64
        ));
    }
}
