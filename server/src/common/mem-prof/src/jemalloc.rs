// SPDX-License-Identifier: GNU GENERAL PUBLIC LICENSE Version 3
//
// Copyleft (c) 2024 James Wong. This file is part of James Wong.
// is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// James Wong is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with James Wong.  If not, see <https://www.gnu.org/licenses/>.
//
// IMPORTANT: Any software that fully or partially contains or uses materials
// covered by this license must also be released under the GNU GPL license.
// This includes modifications and derived works.

mod error;

use std::ffi::{ c_char, CString };
use std::path::PathBuf;

use error::{
    BuildTempPathSnafu,
    DumpProfileDataSnafu,
    OpenTempFileSnafu,
    ProfilingNotEnabledSnafu,
    ReadOptProfSnafu,
};
use snafu::{ ensure, ResultExt };
use tokio::io::AsyncReadExt;

use crate::error::Result;

const PROF_DUMP: &[u8] = b"prof.dump\0";
const OPT_PROF: &[u8] = b"opt.prof\0";

pub async fn dump_profile() -> Result<Vec<u8>> {
    ensure!(is_prof_enabled()?, ProfilingNotEnabledSnafu);
    let tmp_path = tempfile::tempdir().map_err(|_| {
        (BuildTempPathSnafu {
            path: std::env::temp_dir(),
        }).build()
    })?;

    let mut path_buf = PathBuf::from(tmp_path.path());
    path_buf.push("greptimedb.hprof");

    let path = path_buf
        .to_str()
        .ok_or_else(|| (BuildTempPathSnafu { path: &path_buf }).build())?
        .to_string();

    let mut bytes = CString::new(path.as_str())
        .map_err(|_| (BuildTempPathSnafu { path: &path_buf }).build())?
        .into_bytes_with_nul();

    {
        // #safety: we always expect a valid temp file path to write profiling data to.
        let ptr = bytes.as_mut_ptr() as *mut c_char;
        unsafe {
            tikv_jemalloc_ctl::raw
                ::write(PROF_DUMP, ptr)
                .context(DumpProfileDataSnafu { path: path_buf })?;
        }
    }

    let mut f = tokio::fs::File
        ::open(path.as_str()).await
        .context(OpenTempFileSnafu { path: &path })?;
    let mut buf = vec![];
    let _ = f.read_to_end(&mut buf).await.context(OpenTempFileSnafu { path })?;
    Ok(buf)
}

fn is_prof_enabled() -> Result<bool> {
    // safety: OPT_PROF variable, if present, is always a boolean value.
    Ok(unsafe { tikv_jemalloc_ctl::raw::read::<bool>(OPT_PROF).context(ReadOptProfSnafu)? })
}
