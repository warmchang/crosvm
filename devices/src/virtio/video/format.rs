// Copyright 2020 The Chromium OS Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Data structures that represent video format information in virtio video devices.

use std::convert::{From, Into, TryFrom};
use std::io;

use data_model::Le32;
use enumn::N;
use sys_util::error;

use crate::virtio::video::command::ReadCmdError;
use crate::virtio::video::protocol::*;
use crate::virtio::video::response::Response;
use crate::virtio::Writer;

#[derive(PartialEq, Eq, PartialOrd, Ord, N, Clone, Copy, Debug)]
#[repr(u32)]
pub enum Profile {
    H264Baseline = VIRTIO_VIDEO_PROFILE_H264_BASELINE,
    H264Main = VIRTIO_VIDEO_PROFILE_H264_MAIN,
    H264Extended = VIRTIO_VIDEO_PROFILE_H264_EXTENDED,
    H264High = VIRTIO_VIDEO_PROFILE_H264_HIGH,
    H264High10 = VIRTIO_VIDEO_PROFILE_H264_HIGH10PROFILE,
    H264High422 = VIRTIO_VIDEO_PROFILE_H264_HIGH422PROFILE,
    H264High444PredictiveProfile = VIRTIO_VIDEO_PROFILE_H264_HIGH444PREDICTIVEPROFILE,
    H264ScalableBaseline = VIRTIO_VIDEO_PROFILE_H264_SCALABLEBASELINE,
    H264ScalableHigh = VIRTIO_VIDEO_PROFILE_H264_SCALABLEHIGH,
    H264StereoHigh = VIRTIO_VIDEO_PROFILE_H264_STEREOHIGH,
    H264MultiviewHigh = VIRTIO_VIDEO_PROFILE_H264_MULTIVIEWHIGH,
    HevcMain = VIRTIO_VIDEO_PROFILE_HEVC_MAIN,
    HevcMain10 = VIRTIO_VIDEO_PROFILE_HEVC_MAIN10,
    HevcMainStillPicture = VIRTIO_VIDEO_PROFILE_HEVC_MAIN_STILL_PICTURE,
    VP8Profile0 = VIRTIO_VIDEO_PROFILE_VP8_PROFILE0,
    VP8Profile1 = VIRTIO_VIDEO_PROFILE_VP8_PROFILE1,
    VP8Profile2 = VIRTIO_VIDEO_PROFILE_VP8_PROFILE2,
    VP8Profile3 = VIRTIO_VIDEO_PROFILE_VP8_PROFILE3,
    VP9Profile0 = VIRTIO_VIDEO_PROFILE_VP9_PROFILE0,
    VP9Profile1 = VIRTIO_VIDEO_PROFILE_VP9_PROFILE1,
    VP9Profile2 = VIRTIO_VIDEO_PROFILE_VP9_PROFILE2,
    VP9Profile3 = VIRTIO_VIDEO_PROFILE_VP9_PROFILE3,
}
impl_try_from_le32_for_enumn!(Profile, "profile");

macro_rules! impl_libvda_conversion {
    ( $( ( $x:ident, $y:ident ) ),* ) => {
        pub fn from_libvda_profile(p: libvda::Profile) -> Option<Self> {
            match p {
                $(libvda::Profile::$x => Some(Self::$y),)*
                _ => None
            }
        }

        // TODO(alexlau): Remove this after encoder CL lands.
        #[allow(dead_code)]
        pub fn to_libvda_profile(&self) -> Option<libvda::Profile> {
            match self {
                $(Self::$y => Some(libvda::Profile::$x),)*
                _ => None
            }
        }
    }
}

impl Profile {
    pub fn to_format(&self) -> Format {
        use Profile::*;
        match self {
            H264Baseline
            | H264Main
            | H264Extended
            | H264High
            | H264High10
            | H264High422
            | H264High444PredictiveProfile
            | H264ScalableBaseline
            | H264ScalableHigh
            | H264StereoHigh
            | H264MultiviewHigh => Format::H264,
            HevcMain | HevcMain10 | HevcMainStillPicture => Format::HEVC,
            VP8Profile0 | VP8Profile1 | VP8Profile2 | VP8Profile3 => Format::VP8,
            VP9Profile0 | VP9Profile1 | VP9Profile2 | VP9Profile3 => Format::VP9,
        }
    }

    impl_libvda_conversion!(
        (H264ProfileBaseline, H264Baseline),
        (H264ProfileMain, H264Main),
        (H264ProfileExtended, H264Extended),
        (H264ProfileHigh, H264High),
        (H264ProfileHigh10Profile, H264High10),
        (H264ProfileHigh422Profile, H264High422),
        (
            H264ProfileHigh444PredictiveProfile,
            H264High444PredictiveProfile
        ),
        (H264ProfileScalableBaseline, H264ScalableBaseline),
        (H264ProfileScalableHigh, H264ScalableHigh),
        (H264ProfileStereoHigh, H264StereoHigh),
        (H264ProfileMultiviewHigh, H264MultiviewHigh),
        (HevcProfileMain, HevcMain),
        (HevcProfileMain10, HevcMain10),
        (HevcProfileMainStillPicture, HevcMainStillPicture),
        (VP8, VP8Profile0),
        (VP9Profile0, VP9Profile0),
        (VP9Profile1, VP9Profile1),
        (VP9Profile2, VP9Profile2),
        (VP9Profile3, VP9Profile3)
    );
}

#[derive(PartialEq, Eq, PartialOrd, Ord, N, Clone, Copy, Debug)]
#[repr(u32)]
pub enum Level {
    H264_1_0 = VIRTIO_VIDEO_LEVEL_H264_1_0,
}
impl_try_from_le32_for_enumn!(Level, "level");

#[derive(PartialEq, Eq, PartialOrd, Ord, N, Clone, Copy, Debug)]
#[repr(u32)]
pub enum Format {
    // Raw formats
    NV12 = VIRTIO_VIDEO_FORMAT_NV12,
    YUV420 = VIRTIO_VIDEO_FORMAT_YUV420,

    // Bitstream formats
    H264 = VIRTIO_VIDEO_FORMAT_H264,
    HEVC = VIRTIO_VIDEO_FORMAT_HEVC,
    VP8 = VIRTIO_VIDEO_FORMAT_VP8,
    VP9 = VIRTIO_VIDEO_FORMAT_VP9,
}
impl_try_from_le32_for_enumn!(Format, "format");

#[derive(Debug, Default, Copy, Clone)]
pub struct Crop {
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
}
impl_from_for_interconvertible_structs!(virtio_video_crop, Crop, left, top, width, height);

#[derive(Debug, Default, Clone, Copy)]
pub struct PlaneFormat {
    pub plane_size: u32,
    pub stride: u32,
}
impl_from_for_interconvertible_structs!(virtio_video_plane_format, PlaneFormat, plane_size, stride);

#[derive(Debug, Default, Clone, Copy)]
pub struct FormatRange {
    pub min: u32,
    pub max: u32,
    pub step: u32,
}
impl_from_for_interconvertible_structs!(virtio_video_format_range, FormatRange, min, max, step);

#[derive(Debug, Default, Clone)]
pub struct FrameFormat {
    pub width: FormatRange,
    pub height: FormatRange,
    pub bitrates: Vec<FormatRange>,
}

impl Response for FrameFormat {
    fn write(&self, w: &mut Writer) -> Result<(), io::Error> {
        w.write_obj(virtio_video_format_frame {
            width: self.width.into(),
            height: self.height.into(),
            num_rates: Le32::from(self.bitrates.len() as u32),
            ..Default::default()
        })?;
        w.write_iter(
            self.bitrates
                .iter()
                .map(|r| Into::<virtio_video_format_range>::into(*r)),
        )
    }
}

#[derive(Debug, Clone)]
pub struct FormatDesc {
    pub mask: u64,
    pub format: Format,
    pub frame_formats: Vec<FrameFormat>,
}

impl Response for FormatDesc {
    fn write(&self, w: &mut Writer) -> Result<(), io::Error> {
        w.write_obj(virtio_video_format_desc {
            mask: self.mask.into(),
            format: Le32::from(self.format as u32),
            // ChromeOS only supports single-buffer mode.
            planes_layout: Le32::from(VIRTIO_VIDEO_PLANES_LAYOUT_SINGLE_BUFFER),
            // No alignment is required on boards that we currently support.
            plane_align: Le32::from(0),
            num_frames: Le32::from(self.frame_formats.len() as u32),
        })?;
        self.frame_formats.iter().map(|ff| ff.write(w)).collect()
    }
}
