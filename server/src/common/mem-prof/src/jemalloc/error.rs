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

use std::any::Any;
use std::path::PathBuf;

use common_error::ext::{ BoxedError, ErrorExt };
use common_error::status_code::StatusCode;
use common_macro::stack_trace_debug;
use snafu::{ Location, Snafu };

#[derive(Snafu)]
#[snafu(visibility(pub))]
#[stack_trace_debug]
pub enum Error {
    #[snafu(display("Failed to read OPT_PROF"))] ReadOptProf {
        #[snafu(source)]
        error: tikv_jemalloc_ctl::Error,
    },

    #[snafu(display("Memory profiling is not enabled"))]
    ProfilingNotEnabled,

    #[snafu(display("Failed to build temp file from given path: {:?}", path))] BuildTempPath {
        path: PathBuf,
        location: Location,
    },

    #[snafu(display("Failed to open temp file: {}", path))] OpenTempFile {
        path: String,
        #[snafu(source)]
        error: std::io::Error,
    },

    #[snafu(display("Failed to dump profiling data to temp file: {:?}", path))] DumpProfileData {
        path: PathBuf,
        #[snafu(source)]
        error: tikv_jemalloc_ctl::Error,
    },
}

impl ErrorExt for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::ReadOptProf { .. } => StatusCode::Internal,
            Error::ProfilingNotEnabled => StatusCode::InvalidArguments,
            Error::BuildTempPath { .. } => StatusCode::Internal,
            Error::OpenTempFile { .. } => StatusCode::StorageUnavailable,
            Error::DumpProfileData { .. } => StatusCode::StorageUnavailable,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl From<Error> for crate::error::Error {
    fn from(e: Error) -> Self {
        Self::Internal {
            source: BoxedError::new(e),
        }
    }
}
