fn main() {
    println!("cargo:rerun-if-env-changed=PYLON_VERSION");
    println!("cargo:rerun-if-env-changed=PYLON_ROOT");
    println!("cargo:rerun-if-env-changed=PYLON_DEV_DIR");
    println!("cargo:rerun-if-env-changed=PYLONFRAMEWORKDIR");

    let pylon_major_version: Option<u8> =
        std::env::var_os("PYLON_VERSION").map(|s| s.into_string().unwrap().parse::<u8>().unwrap());

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=include/catcher.h");
    println!("cargo:rerun-if-changed=include/pylon-cxx-rs.h");
    println!("cargo:rerun-if-changed=src/pylon-cxx-rs.cc");

    let mut build = cxx_build::bridge("src/lib.rs");

    build
        .file("src/pylon-cxx-rs.cc")
        .warnings(false)
        .cpp(true)
        .include("include");

    #[cfg(all(not(target_os = "windows"), feature = "stream"))]
    build.define("FEATURE_STREAM_UNIX", None);

    #[cfg(all(target_os = "windows", feature = "stream"))]
    build.define("FEATURE_STREAM_WINDOWS", None);

    #[cfg(target_os = "linux")]
    {
        let pylon_root = match std::env::var("PYLON_ROOT") {
            Ok(val) => val,
            Err(_) => match pylon_major_version {
                Some(5) => "/opt/pylon5",
                Some(6) | None => "/opt/pylon",
                Some(version) => panic!("unsupported pylon version: {}", version),
            }
            .into(),
        };

        let expected_major_version = match pylon_root.as_str() {
            "/opt/pylon5" => Some(5),
            "/opt/pylon" => Some(6),
            _ => None,
        };

        let pylon_major_version = match (expected_major_version, pylon_major_version) {
            (Some(expected_major_version), Some(actual_major_version)) => {
                assert_eq!(expected_major_version, actual_major_version);
                actual_major_version
            }
            (Some(v), None) | (None, Some(v)) => v,
            (None, None) => 6,
        };

        let pylon_root = std::path::PathBuf::from(pylon_root);

        let include1 = pylon_root.join("include");

        build.flag("-std=c++14").include(&include1);

        let mut lib_dir = pylon_root;
        if pylon_major_version == 5 {
            lib_dir.push("lib64");
        } else {
            lib_dir.push("lib");
        }

        let dir_str = lib_dir.to_str().unwrap();

        println!("cargo:rustc-link-search=native={dir_str}");
        println!("cargo:rustc-link-lib=pylonc");

        // The Basler docs want the rest of these libraries to be automatically
        // found using rpath linker args, but sending options to the linker in rust
        // requires the unstable link_args feature. So we specify them manually.
        // See https://github.com/rust-lang/cargo/issues/5077
        println!("cargo:rustc-link-lib=pylonbase");
        println!("cargo:rustc-link-lib=pylonutility");
        println!("cargo:rustc-link-lib=gxapi");

        if pylon_major_version == 5 {
            enum PylonVersion {
                V5_0,
                V5_1,
                V5_2,
                Unknown,
            }

            let mut so_file_for_5_2 = lib_dir.clone();
            so_file_for_5_2.push("libpylon_TL_usb-5.2.0.so");

            let mut so_file_for_5_1 = lib_dir.clone();
            so_file_for_5_1.push("libGenApi_gcc_v3_1_Basler_pylon_v5_1");
            so_file_for_5_1.set_extension("so");

            eprint!(
                "# pylon build: checking for file {}...",
                so_file_for_5_2.display()
            );
            let version = if so_file_for_5_2.exists() {
                eprintln!("found");
                PylonVersion::V5_2
            } else {
                eprintln!("not found");

                eprint!(
                    "# pylon build: checking for file {}...",
                    so_file_for_5_1.display()
                );
                if so_file_for_5_1.exists() {
                    eprintln!("found");
                    PylonVersion::V5_1
                } else {
                    eprintln!("not found");
                    let mut so_file_for_5_0 = lib_dir.clone();
                    so_file_for_5_0.push("libGenApi_gcc_v3_0_Basler_pylon_v5_0");
                    so_file_for_5_0.set_extension("so");
                    eprint!(
                        "# pylon build: checking for file {}...",
                        so_file_for_5_0.display()
                    );
                    if so_file_for_5_0.exists() {
                        eprintln!("found");
                        PylonVersion::V5_0
                    } else {
                        eprintln!("not found");
                        PylonVersion::Unknown
                    }
                }
            };

            match version {
                PylonVersion::V5_0 => {
                    println!("cargo:rustc-link-lib=GenApi_gcc_v3_0_Basler_pylon_v5_0");
                    println!("cargo:rustc-link-lib=GCBase_gcc_v3_0_Basler_pylon_v5_0");
                    println!("cargo:rustc-link-lib=Log_gcc_v3_0_Basler_pylon_v5_0");
                    println!("cargo:rustc-link-lib=MathParser_gcc_v3_0_Basler_pylon_v5_0");
                    println!("cargo:rustc-link-lib=XmlParser_gcc_v3_0_Basler_pylon_v5_0");
                    println!("cargo:rustc-link-lib=NodeMapData_gcc_v3_0_Basler_pylon_v5_0");
                }
                PylonVersion::V5_1 => {
                    println!("cargo:rustc-link-lib=GenApi_gcc_v3_1_Basler_pylon_v5_1");
                    println!("cargo:rustc-link-lib=GCBase_gcc_v3_1_Basler_pylon_v5_1");
                    println!("cargo:rustc-link-lib=Log_gcc_v3_1_Basler_pylon_v5_1");
                    println!("cargo:rustc-link-lib=MathParser_gcc_v3_1_Basler_pylon_v5_1");
                    println!("cargo:rustc-link-lib=XmlParser_gcc_v3_1_Basler_pylon_v5_1");
                    println!("cargo:rustc-link-lib=NodeMapData_gcc_v3_1_Basler_pylon_v5_1");
                }
                PylonVersion::V5_2 => {
                    println!("cargo:rustc-link-lib=GenApi_gcc_v3_1_Basler_pylon");
                    println!("cargo:rustc-link-lib=GCBase_gcc_v3_1_Basler_pylon");
                    println!("cargo:rustc-link-lib=Log_gcc_v3_1_Basler_pylon");
                    println!("cargo:rustc-link-lib=MathParser_gcc_v3_1_Basler_pylon");
                    println!("cargo:rustc-link-lib=XmlParser_gcc_v3_1_Basler_pylon");
                    println!("cargo:rustc-link-lib=NodeMapData_gcc_v3_1_Basler_pylon");
                }
                PylonVersion::Unknown => {
                    panic!("could not detect pylon library version");
                }
            }
        } else {
            assert_eq!(pylon_major_version, 6);

            let lib_names = [
                "GenApi", "GCBase", "Log", "MathParser", "XmlParser", "NodeMapData"
            ];

            let common_version = find_common_lib_version_linux(&lib_dir, &lib_names)
                .unwrap_or_else(|| panic!(
                    "could not find common library version for all required libraries: {}",
                    lib_dir.display()
                ));

            eprintln!("INFO - using common pylon library version: {common_version}");

            for lib_name in &lib_names {
                let link_name = format!("{lib_name}_{common_version}");
                eprintln!("INFO - found pylon library: {link_name}");
                println!("cargo:rustc-link-lib={link_name}");
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        match pylon_major_version {
            Some(6) | Some(7) | None => {
                // directory with `pylon.framework`
                let pylon_framework_dir = match std::env::var_os("PYLONFRAMEWORKDIR") {
                    Some(val) => std::path::PathBuf::from(val),
                    None => "/Library/Frameworks".into(),
                };

                assert!(
                    pylon_major_version == Some(6)
                        || pylon_major_version == Some(7)
                        || pylon_major_version.is_none()
                );

                let flag = format!("-F{}", pylon_framework_dir.display());

                println!("cargo:rustc-link-arg=-Wl,-ld_classic");

                let lib_dir = pylon_framework_dir.join("pylon.framework/Libraries");
                println!("cargo:rustc-link-search={}", lib_dir.display());
                println!("cargo:rustc-link-lib=pylonbase");
                println!("cargo:rustc-link-lib=pylonutility");
                println!("cargo:rustc-link-lib=GenApi_gcc_v3_1_Basler_pylon");
                println!("cargo:rustc-link-lib=GCBase_gcc_v3_1_Basler_pylon");
                println!("cargo:rustc-link-lib=Log_gcc_v3_1_Basler_pylon");
                println!("cargo:rustc-link-lib=MathParser_gcc_v3_1_Basler_pylon");
                println!("cargo:rustc-link-lib=XmlParser_gcc_v3_1_Basler_pylon");
                println!("cargo:rustc-link-lib=NodeMapData_gcc_v3_1_Basler_pylon");

                build
                    .flag("-std=c++14")
                    .include(pylon_framework_dir.join("pylon.framework/Versions/A/Headers/GenICam"))
                    .flag(&flag);
            }
            Some(version) => panic!("unsupported pylon version: {}", version),
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::path::PathBuf;

        let pylon_dev_dir = match std::env::var_os("PYLON_DEV_DIR") {
            Some(val) => PathBuf::from(val),
            None => match pylon_major_version {
                Some(5) => PathBuf::from(r#"C:\Program Files\Basler\pylon 5\Development"#),
                Some(6) | None => PathBuf::from(r#"C:\Program Files\Basler\pylon 6\Development"#),
                Some(version) => panic!("unsupported pylon version: {}", version),
            },
        };

        let mut include_dir = pylon_dev_dir.clone();
        include_dir.push("include");

        let mut pylon_include_dir = include_dir.clone();
        pylon_include_dir.push("pylon");

        let mut lib_dir = pylon_dev_dir;
        lib_dir.push("lib");
        lib_dir.push("x64");

        println!("cargo:rustc-link-search={}", lib_dir.display());

        build.include(include_dir);
    }

    build.compile("pyloncxxrs");
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PylonLibVersion {
    gcc_major: u8,
    gcc_minor: u8,
    pylon_suffix: String,
}

impl Ord for PylonLibVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.gcc_major.cmp(&other.gcc_major) {
            std::cmp::Ordering::Equal => {
                match self.gcc_minor.cmp(&other.gcc_minor) {
                    std::cmp::Ordering::Equal => compare_pylon_suffix(&self.pylon_suffix, &other.pylon_suffix),
                    ord => ord,
                }
            },
            ord => ord,
        }
    }
}

impl PartialOrd for PylonLibVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn compare_pylon_suffix(a: &str, b: &str) -> std::cmp::Ordering {
    if a == b {
        return std::cmp::Ordering::Equal;
    }

    if a.is_empty() {
        return std::cmp::Ordering::Less;
    }

    if b.is_empty() {
        return std::cmp::Ordering::Greater;
    }

    let parse_suffix = |s: &str| -> Vec<u8> {
        s.trim_start_matches('v')
            .split('_')
            .filter_map(|n| n.parse::<u8>().ok())
            .collect()
    };

    let a_parts = parse_suffix(a);
    let b_parts = parse_suffix(b);

    for i in 0..std::cmp::max(a_parts.len(), b_parts.len()) {
        let a_val = a_parts.get(i).copied().unwrap_or(0);
        let b_val = b_parts.get(i).copied().unwrap_or(0);

        match a_val.cmp(&b_val) {
            std::cmp::Ordering::Equal => continue,
            ord => return ord,
        }
    }

    std::cmp::Ordering::Equal
}

impl std::fmt::Display for PylonLibVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.pylon_suffix.is_empty() {
            write!(f, "gcc_v{}_{}_Basler_pylon", self.gcc_major, self.gcc_minor)
        } else {
            write!(f, "gcc_v{}_{}_Basler_pylon_{}", self.gcc_major, self.gcc_minor, self.pylon_suffix)
        }
    }
}

fn find_common_lib_version_linux(lib_dir: &std::path::Path, lib_names: &[&str]) -> Option<String> {
    let mut versions_by_lib: Vec<Vec<PylonLibVersion>> = Vec::new();

    for lib_name in lib_names {
        let versions = find_all_lib_versions_linux(lib_dir, lib_name);

        if versions.is_empty() {
            eprintln!("WARNING - no versions found for {lib_name}");
            return None;
        }

        versions_by_lib.push(versions);
    }

    let first_lib_versions = &versions_by_lib[0];
    let mut common_versions: Vec<PylonLibVersion> = Vec::new();

    for version in first_lib_versions {
        let exists_in_all = versions_by_lib[1..].iter().all(|lib_versions| {
            lib_versions.contains(version)
        });

        if exists_in_all {
            common_versions.push(version.clone());
        }
    }

    eprintln!("INFO - available pylon versions by library:");
    for (i, lib_name) in lib_names.iter().enumerate() {
        eprintln!("\t{lib_name}: {:?}", versions_by_lib[i]);
    }

    if common_versions.is_empty() {
        eprintln!("ERROR - no common version found across all libraries");
        return None;
    }

    common_versions.sort();
    common_versions.reverse();

    Some(common_versions[0].to_string())
}

fn find_all_lib_versions_linux(lib_dir: &std::path::Path, lib_name: &str) -> Vec<PylonLibVersion> {
    let mut versions = Vec::new();

    if let Ok(entries) = std::fs::read_dir(lib_dir) {
        // names start with lib<lib_name>_gcc_v*
        let lib_prefix = format!("lib{lib_name}_gcc_v");

        for entry in entries.flatten() {
            let filename = entry.file_name();
            let filename = filename.to_str().unwrap_or("");

            // try to match pattern: lib<lib_name>_gcc_v<M>_<m>_Basler_pylon(_v<n>).so
            if filename.starts_with(&lib_prefix) && filename.contains("_Basler_pylon") && filename.ends_with(".so") {
                if let Some(version) = parse_lib_version_linux(filename, &lib_prefix) {
                    versions.push(version);
                }
            }
        }
    }

    versions
}

fn parse_lib_version_linux(filename: &str, prefix: &str) -> Option<PylonLibVersion> {
    let version_part = filename
        .strip_prefix(prefix)?
        .strip_suffix(".so")?;

    let parts: Vec<&str> = version_part.split("_Basler_pylon").collect();
    if parts.is_empty() {
        return None;
    }

    let gcc_parts: Vec<&str> = parts[0].split('_').collect();
    if gcc_parts.len() < 2 {
        return None;
    }

    // extract GCC major/minor versions
    let gcc_major = gcc_parts[0].parse::<u8>().ok()?;
    let gcc_minor = gcc_parts[1].parse::<u8>().ok()?;
    
    // if there is a suffix version, extract that too
    let pylon_suffix = if parts.len() > 1 && !parts[1].is_empty() {
        parts[1].trim_start_matches('_').to_string()
    } else {
        String::new()
    };

    Some(PylonLibVersion { gcc_major, gcc_minor, pylon_suffix })
}