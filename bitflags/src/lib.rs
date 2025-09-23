use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FilePermissions: u32 {
        const READ = 0b0000_0001;
        const WRITE = 0b0000_0010;
        const EXECUTE = 0b0000_0100;
        const READ_WRITE = Self::READ.bits() | Self::WRITE.bits();
        const ALL = Self::READ.bits() | Self::WRITE.bits() | Self::EXECUTE.bits();
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CompilerFlags: u32 {
        const OPTIMIZE = 1 << 0;
        const DEBUG_INFO = 1 << 1;
        const WARNINGS_AS_ERRORS = 1 << 2;
        const VERBOSE = 1 << 3;
        const LINK_TIME_OPTIMIZATION = 1 << 4;
        const STATIC_LINKING = 1 << 5;
        const PROFILE = 1 << 6;
        const RELEASE = Self::OPTIMIZE.bits() | Self::LINK_TIME_OPTIMIZATION.bits();
        const DEBUG = Self::DEBUG_INFO.bits() | Self::VERBOSE.bits();
    }
}

pub fn demonstrate_file_permissions() {
    let mut perms = FilePermissions::READ | FilePermissions::WRITE;
    println!("Initial permissions: {:?}", perms);
    println!("Can read: {}", perms.contains(FilePermissions::READ));
    println!("Can execute: {}", perms.contains(FilePermissions::EXECUTE));

    perms.insert(FilePermissions::EXECUTE);
    println!("After adding execute: {:?}", perms);

    perms.remove(FilePermissions::WRITE);
    println!("After removing write: {:?}", perms);

    let readonly = FilePermissions::READ;
    let readwrite = FilePermissions::READ_WRITE;
    println!(
        "Read-only intersects with read-write: {}",
        !readonly.intersection(readwrite).is_empty()
    );
}

pub fn demonstrate_compiler_flags() {
    let debug_build = CompilerFlags::DEBUG;
    let release_build = CompilerFlags::RELEASE;

    println!("Debug flags: {:?}", debug_build);
    println!("Release flags: {:?}", release_build);

    let custom =
        CompilerFlags::OPTIMIZE | CompilerFlags::DEBUG_INFO | CompilerFlags::WARNINGS_AS_ERRORS;
    println!("Custom build flags: {:?}", custom);
    println!(
        "Custom has optimization: {}",
        custom.contains(CompilerFlags::OPTIMIZE)
    );

    let common = debug_build.intersection(release_build);
    println!("Common flags between debug and release: {:?}", common);
}

#[derive(Debug)]
pub struct CompilerOptions {
    flags: CompilerFlags,
    optimization_level: u8,
}

impl CompilerOptions {
    pub fn new(flags: CompilerFlags) -> Self {
        let optimization_level = if flags.contains(CompilerFlags::OPTIMIZE) {
            if flags.contains(CompilerFlags::LINK_TIME_OPTIMIZATION) {
                3
            } else {
                2
            }
        } else {
            0
        };

        Self {
            flags,
            optimization_level,
        }
    }

    pub fn is_debug_build(&self) -> bool {
        self.flags.contains(CompilerFlags::DEBUG_INFO)
    }

    pub fn enable_profiling(&mut self) {
        self.flags.insert(CompilerFlags::PROFILE);
    }

    pub fn optimization_level(&self) -> u8 {
        self.optimization_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_permissions() {
        let perms = FilePermissions::READ | FilePermissions::WRITE;
        assert!(perms.contains(FilePermissions::READ));
        assert!(perms.contains(FilePermissions::WRITE));
        assert!(!perms.contains(FilePermissions::EXECUTE));

        assert_eq!(
            FilePermissions::ALL,
            FilePermissions::READ | FilePermissions::WRITE | FilePermissions::EXECUTE
        );
    }

    #[test]
    fn test_compiler_flags() {
        let debug = CompilerFlags::DEBUG;
        assert!(debug.contains(CompilerFlags::DEBUG_INFO));
        assert!(debug.contains(CompilerFlags::VERBOSE));
        assert!(!debug.contains(CompilerFlags::OPTIMIZE));
    }

    #[test]
    fn test_compiler_options() {
        let release_options = CompilerOptions::new(CompilerFlags::RELEASE);
        assert!(!release_options.is_debug_build());
        assert_eq!(release_options.optimization_level, 3);

        let debug_options = CompilerOptions::new(CompilerFlags::DEBUG);
        assert!(debug_options.is_debug_build());
        assert_eq!(debug_options.optimization_level, 0);
    }
}
