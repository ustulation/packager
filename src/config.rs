use std::path::PathBuf;

#[derive(Debug, RustcDecodable, Clone)]
pub struct Config {
    output_dir: String,
    demo_app: Option<DemoApp>,
    safe_launcher: Option<SafeLauncher>,
}

#[derive(Debug)]
pub struct ConfigImpl {
    pub output_dir: PathBuf,
    pub demo_app: Option<DemoAppImpl>,
    pub safe_launcher: Option<SafeLauncherImpl>,
}

impl Into<ConfigImpl> for Config {
    fn into(self) -> ConfigImpl {
        ConfigImpl {
            output_dir: PathBuf::from(self.output_dir),
            demo_app: self.demo_app.map(|v| v.into()),
            safe_launcher: self.safe_launcher.map(|v| v.into()),
        }
    }
}

#[derive(Debug, RustcDecodable, Clone)]
pub struct DemoApp {
    demo_app_project_dir: String,
    build: bool,
}

#[derive(Debug)]
pub struct DemoAppImpl {
    pub demo_app_project_dir: PathBuf,
    pub build: bool,
}

impl Into<DemoAppImpl> for DemoApp {
    fn into(self) -> DemoAppImpl {
        DemoAppImpl {
            demo_app_project_dir: PathBuf::from(self.demo_app_project_dir),
            build: self.build,
        }
    }
}

#[derive(Debug, RustcDecodable, Clone)]
pub struct SafeLauncher {
    launcher_project_dir: String,
    log_toml_path: String,
    crust_config_path: String,
    safe_core: SafeCore,
}

#[derive(Debug)]
pub struct SafeLauncherImpl {
    pub launcher_project_dir: PathBuf,
    pub log_toml_path: PathBuf,
    pub crust_config_path: PathBuf,
    pub safe_core: SafeCoreImpl,
}

impl Into<SafeLauncherImpl> for SafeLauncher {
    fn into(self) -> SafeLauncherImpl {
        SafeLauncherImpl {
            launcher_project_dir: PathBuf::from(self.launcher_project_dir),
            log_toml_path: PathBuf::from(self.log_toml_path),
            crust_config_path: PathBuf::from(self.crust_config_path),
            safe_core: self.safe_core.into(),
        }
    }
}

#[derive(Debug, RustcDecodable, Clone)]
pub struct SafeCore {
    project_dir: String,
    relative_target_dir: Option<String>,
    use_nightly: bool,
    clean: bool,
    release_build: bool,
    run_tests: bool,
}

#[derive(Debug)]
pub struct SafeCoreImpl {
    pub project_dir: PathBuf,
    pub relative_target_dir: Option<PathBuf>,
    pub use_nightly: bool,
    pub clean: bool,
    pub release_build: bool,
    pub run_tests: bool,
}

impl Into<SafeCoreImpl> for SafeCore {
    fn into(self) -> SafeCoreImpl {
        SafeCoreImpl {
            project_dir: PathBuf::from(self.project_dir),
            relative_target_dir: self.relative_target_dir.map(PathBuf::from),
            use_nightly: self.use_nightly,
            clean: self.clean,
            release_build: self.release_build,
            run_tests: self.run_tests,
        }
    }
}
