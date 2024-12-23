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

#![feature(error_iter)]

pub mod ext;
pub mod format;
pub mod mock;
pub mod status_code;

pub use snafu;

// HACK - these headers are here for shared in gRPC services. For common HTTP headers,
// please define in `src/servers/src/http/header.rs`.
pub const GREPTIME_DB_HEADER_ERROR_CODE: &str = "x-greptime-err-code";
pub const GREPTIME_DB_HEADER_ERROR_MSG: &str = "x-greptime-err-msg";
