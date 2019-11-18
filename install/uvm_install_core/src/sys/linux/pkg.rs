use crate::*;
use std::ffi::OsStr;
use std::fs::DirBuilder;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub type ModulePkgInstaller = Installer<UnityModule, Pkg, InstallerWithDestination>;

impl ModulePkgInstaller {
    fn xar<P, D>(&self, installer: P, destination: D) -> Result<()>
    where
        P: AsRef<Path>,
        D: AsRef<Path>,
    {
        let installer = installer.as_ref();
        let destination = destination.as_ref();

        debug!(
            "unpack installer {} to temp destination {}",
            installer.display(),
            destination.display()
        );

        let child = Command::new("7z")
            .arg("x")
            .arg("-y")
            .arg(format!("-o{}", destination.display()))
            .arg(installer)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let output = child.wait_with_output()?;
        if !output.status.success() {
            return Err(format!(
                "failed to extract installer:/n{}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        Ok(())
    }

    fn untar<P, D>(&self, base_payload_path: P, destination: D) -> Result<()>
    where
        P: AsRef<Path>,
        D: AsRef<Path>,
    {
        let base_payload_path = base_payload_path.as_ref();
        let destination = destination.as_ref();

        let payload = self.find_payload(base_payload_path)?;
        debug!("extract payload at {}", payload.display());

        let tar_child = if payload.file_name() == Some(OsStr::new("Payload~")) {
            let mut cpio = Command::new("cpio")
                .arg("-iu")
                .current_dir(destination)
                .stdin(Stdio::piped())
                .spawn()?;
            {
                let stdin = cpio.stdin.as_mut().expect("stdin");
                let mut file = File::open(payload)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                stdin.write(&buffer)?;
            }
            cpio
        } else {
            let mut gzip = Command::new("gzip")
                .arg("-dc")
                .arg(payload)
                .stdout(Stdio::piped())
                .spawn()?;

            let mut cpio = Command::new("cpio")
                .arg("-iu")
                .current_dir(destination)
                .stdin(Stdio::piped())
                .spawn()?;
            {
                let stdin = cpio.stdin.as_mut().expect("stdin");
                let gzip_std_out = gzip.stdout.as_mut().expect("stdout");
                let mut buffer = Vec::new();
                gzip_std_out.read_to_end(&mut buffer)?;
                stdin.write(&buffer)?;
            }
            cpio
        };

        let tar_output = tar_child.wait_with_output()?;
        if !tar_output.status.success() {
            return Err(format!(
                "failed to untar payload:/n{}",
                String::from_utf8_lossy(&tar_output.stderr)
            )
            .into());
        }

        Ok(())
    }
}

impl InstallHandler for ModulePkgInstaller {
    fn install_handler(&self) -> Result<()> {
        let destination = self.destination();
        let installer = self.installer();

        debug!(
            "install module from pkg {} to {}",
            installer.display(),
            destination.display()
        );

        let tmp_destination = destination.join("tmp");
        DirBuilder::new().recursive(true).create(&tmp_destination)?;
        self.xar(installer, &tmp_destination)?;
        self.untar(&tmp_destination, destination)?;
        self.cleanup(&tmp_destination)?;
        Ok(())
    }

    fn after_install(&self) -> Result<()> {
        if let Some((from, to)) = &self.rename() {
            uvm_move_dir::move_dir(from, to).chain_err(|| "failed to rename installed module")?;
        }
        Ok(())
    }

    fn error_handler(&self) {
        self.cleanup_directory_failable(&self.destination());
    }
}
