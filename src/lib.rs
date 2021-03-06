#[macro_use]
extern crate lazy_static;

use ffi::*;

use std::ffi::{CString};
use std::os::raw::{c_char};
use std::slice::{from_raw_parts, from_raw_parts_mut};

pub mod ffi;

lazy_static! {
  static ref MODE_1:        CString = CString::new("1").unwrap();
  static ref MODE_L:        CString = CString::new("L").unwrap();
  static ref MODE_P:        CString = CString::new("P").unwrap();
  static ref MODE_I:        CString = CString::new("I").unwrap();
  static ref MODE_F:        CString = CString::new("F").unwrap();
  static ref MODE_RGB:      CString = CString::new("RGB").unwrap();
  static ref MODE_RGBA:     CString = CString::new("RGBA").unwrap();
  static ref MODE_RGBX:     CString = CString::new("RGBX").unwrap();
  static ref MODE_CMYK:     CString = CString::new("CMYK").unwrap();
  static ref MODE_YCBCR:    CString = CString::new("YCbCr").unwrap();
  static ref MODE_LAB:      CString = CString::new("LAB").unwrap();
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PILType {
  Uint8,
  Int32,
  Float32,
}

impl PILType {
  pub fn from_raw(raw: u32) -> Self {
    match raw {
      IMAGING_TYPE_UINT8    => PILType::Uint8,
      IMAGING_TYPE_INT32    => PILType::Int32,
      IMAGING_TYPE_FLOAT32  => PILType::Float32,
      IMAGING_TYPE_SPECIAL  => unimplemented!(),
      _ => unreachable!(),
    }
  }

  pub fn to_raw(&self) -> u32 {
    match *self {
      PILType::Uint8    => IMAGING_TYPE_UINT8,
      PILType::Int32    => IMAGING_TYPE_INT32,
      PILType::Float32  => IMAGING_TYPE_FLOAT32,
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PILMode {
  Unit,
  L,
  P,
  I,
  F,
  RGB,
  RGBA,
  RGBX,
  CMYK,
  YCbCr,
  LAB,
}

impl PILMode {
  pub fn to_raw(&self) -> *const c_char {
    match *self {
      PILMode::Unit     => MODE_1.as_c_str().as_ptr(),
      PILMode::L        => MODE_L.as_c_str().as_ptr(),
      PILMode::P        => MODE_P.as_c_str().as_ptr(),
      PILMode::I        => MODE_I.as_c_str().as_ptr(),
      PILMode::F        => MODE_F.as_c_str().as_ptr(),
      PILMode::RGB      => MODE_RGB.as_c_str().as_ptr(),
      PILMode::RGBA     => MODE_RGBA.as_c_str().as_ptr(),
      PILMode::RGBX     => MODE_RGBX.as_c_str().as_ptr(),
      PILMode::CMYK     => MODE_CMYK.as_c_str().as_ptr(),
      PILMode::YCbCr    => MODE_YCBCR.as_c_str().as_ptr(),
      PILMode::LAB      => MODE_LAB.as_c_str().as_ptr(),
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PILFilter {
  Nearest,
  Box_,
  Bilinear,
  Hamming,
  Bicubic,
  Lanczos,
}

impl PILFilter {
  pub fn to_raw(&self) -> u32 {
    match *self {
      PILFilter::Nearest    => IMAGING_TRANSFORM_NEAREST,
      PILFilter::Box_       => IMAGING_TRANSFORM_BOX,
      PILFilter::Bilinear   => IMAGING_TRANSFORM_BILINEAR,
      PILFilter::Hamming    => IMAGING_TRANSFORM_HAMMING,
      PILFilter::Bicubic    => IMAGING_TRANSFORM_BICUBIC,
      PILFilter::Lanczos    => IMAGING_TRANSFORM_LANCZOS,
    }
  }
}

pub struct PILImage {
  ptr:  Imaging,
}

unsafe impl Send for PILImage {}
unsafe impl Sync for PILImage {}

impl Drop for PILImage {
  fn drop(&mut self) {
    assert!(!self.ptr.is_null());
    unsafe { ImagingDelete(self.ptr) };
  }
}

impl PILImage {
  pub unsafe fn new(mode: PILMode, xdim: i32, ydim: i32) -> Self {
    //let ptr = unsafe { ImagingNewBlock(mode.to_raw(), xdim, ydim) };
    let ptr = unsafe { ImagingNewDirty(mode.to_raw(), xdim, ydim) };
    assert!(!ptr.is_null());
    PILImage{
      ptr:  ptr,
    }
  }

  pub unsafe fn _new_mode(raw_mode: *const c_char, xdim: i32, ydim: i32) -> Self {
    //let ptr = unsafe { ImagingNewBlock(raw_mode, xdim, ydim) };
    let ptr = unsafe { ImagingNewDirty(raw_mode, xdim, ydim) };
    assert!(!ptr.is_null());
    PILImage{
      ptr:  ptr,
    }
  }

  pub unsafe fn from_raw(raw_im: Imaging) -> Self {
    assert!(!raw_im.is_null());
    PILImage{
      ptr:  raw_im,
    }
  }

  pub unsafe fn as_mut_ptr(&mut self) -> Imaging {
    self.ptr
  }

  pub unsafe fn _raw_mode(&self) -> *const c_char {
    self._mode()
  }

  pub unsafe fn _mode(&self) -> *const c_char {
    assert!(!self.ptr.is_null());
    (*self.ptr).mode.as_ptr()
  }

  pub fn to_vec(&self) -> Vec<u8> {
    let h = self.height();
    let w = self.width();
    let c = self.pixel_channels();
    let px_sz = self.pixel_size_bytes();
    let mut flatv = Vec::with_capacity(self.pixel_channels() as usize * self.width() as usize * self.height() as usize);
    for y in 0 .. h {
      let line = self.raster_line(y);
      for x in 0 .. w {
        for k in 0 .. c {
          flatv.push(line[(px_sz * x + k) as usize]);
        }
      }
    }
    assert_eq!(flatv.len(), self.pixel_channels() as usize * self.width() as usize * self.height() as usize);
    flatv
  }

  pub fn dump_pixels(&self, buf: &mut [u8]) {
    let h = self.height();
    let w = self.width();
    let c = self.pixel_channels();
    let px_sz = self.pixel_size_bytes();
    for y in 0 .. h {
      let line = self.raster_line(y);
      for x in 0 .. w {
        for k in 0 .. c {
          let line_off = (px_sz * x + k) as usize;
          buf[((y * w + x) * c + k) as usize] = line[line_off];
        }
      }
    }
  }

  pub fn dump_planes(&self, buf: &mut [u8]) {
    let h = self.height();
    let w = self.width();
    let c = self.pixel_channels();
    let px_sz = self.pixel_size_bytes();
    for y in 0 .. h {
      let line = self.raster_line(y);
      for x in 0 .. w {
        for k in 0 .. c {
          let line_off = (px_sz * x + k) as usize;
          buf[((k * h + y) * w + x) as usize] = line[line_off];
        }
      }
    }
  }

  pub fn raster_line(&self, y: i32) -> &[u8] {
    assert!(y >= 0);
    assert!(y < self.height());
    assert!(self.line_size_bytes() >= self.pixel_size_bytes() * self.width());
    unsafe { from_raw_parts(*((&*self.ptr).image).offset(y as _) as *mut u8, self.line_size_bytes() as usize) }
  }

  pub fn raster_line_mut(&mut self, y: i32) -> &mut [u8] {
    assert!(y >= 0);
    assert!(y < self.height());
    assert!(self.line_size_bytes() >= self.pixel_size_bytes() * self.width());
    unsafe { from_raw_parts_mut(*((&mut *self.ptr).image).offset(y as _) as *mut u8, self.line_size_bytes() as usize) }
  }

  pub fn width(&self) -> i32 {
    unsafe { (&*self.ptr).xsize }
  }

  pub fn height(&self) -> i32 {
    unsafe { (&*self.ptr).ysize }
  }

  pub fn pixel_type(&self) -> PILType {
    PILType::from_raw(unsafe { (&*self.ptr).type_ } as u32)
  }

  pub fn pixel_channels(&self) -> i32 {
    unsafe { (&*self.ptr).bands }
  }

  pub fn pixel_size_bytes(&self) -> i32 {
    unsafe { (&*self.ptr).pixelsize }
  }

  pub fn line_size_bytes(&self) -> i32 {
    unsafe { (&*self.ptr).linesize }
  }

  pub fn crop(&self, x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
    unsafe { PILImage::from_raw(ImagingCrop(self.ptr, x0, y0, x1, y1)) }
  }

  pub fn flip_left_right(&self, mut out: PILImage) -> Self {
    unsafe { ImagingFlipLeftRight(out.as_mut_ptr(), self.ptr); }
    out
  }

  pub fn resample(&self, new_xdim: i32, new_ydim: i32, filter: PILFilter) -> Self {
    self.resample_crop(new_xdim, new_ydim, filter, [0.0, 0.0, new_xdim as _, new_ydim as _])
  }

  pub fn resample_crop(&self, new_xdim: i32, new_ydim: i32, filter: PILFilter, mut new_crop: [f32; 4]) -> Self {
    unsafe { PILImage::from_raw(ImagingResample(self.ptr, new_xdim, new_ydim, filter.to_raw() as i32, (&mut new_crop).as_mut_ptr())) }
  }
}
