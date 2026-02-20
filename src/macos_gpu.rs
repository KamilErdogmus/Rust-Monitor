//! macOS Apple Silicon GPU monitoring via IOReport private API.
//!
//! This module uses undocumented IOReport functions from IOKit to read
//! GPU utilization, frequency, and power data without requiring sudo.
//! Based on the approach used by macmon and socpowerbud projects.
//!
//! WARNING: This uses private Apple APIs that may break on future macOS versions.

#![allow(non_snake_case, non_upper_case_globals, dead_code)]

use std::ffi::c_void;
use std::ptr;

// ─── Core Foundation types ───────────────────────────────────────────

type CFTypeRef = *const c_void;
type CFDictionaryRef = *const c_void;
type CFMutableDictionaryRef = *mut c_void;
type CFStringRef = *const c_void;
type CFNumberRef = *const c_void;
type CFAllocatorRef = *const c_void;
type CFArrayRef = *const c_void;
type CFIndex = isize;

const kCFAllocatorDefault: CFAllocatorRef = ptr::null();

#[repr(C)]
struct CFRange {
    location: CFIndex,
    length: CFIndex,
}

// ─── IOReport types ──────────────────────────────────────────────────

type IOReportSubscriptionRef = *mut c_void;

extern "C" {
    // Core Foundation
    fn CFRelease(cf: CFTypeRef);
    fn CFArrayGetCount(theArray: CFArrayRef) -> CFIndex;
    fn CFArrayGetValueAtIndex(theArray: CFArrayRef, idx: CFIndex) -> CFTypeRef;
    fn CFDictionaryGetValue(theDict: CFDictionaryRef, key: CFTypeRef) -> CFTypeRef;
    fn CFStringGetLength(theString: CFStringRef) -> CFIndex;
    fn CFStringGetCString(
        theString: CFStringRef,
        buffer: *mut u8,
        bufferSize: CFIndex,
        encoding: u32,
    ) -> bool;
    fn CFNumberGetValue(
        number: CFNumberRef,
        theType: i32,
        valuePtr: *mut c_void,
    ) -> bool;
    fn CFStringCreateWithCString(
        alloc: CFAllocatorRef,
        cStr: *const u8,
        encoding: u32,
    ) -> CFStringRef;

    // IOReport private API (from /usr/lib/libIOReport.dylib)
    fn IOReportCopyChannelsInGroup(
        group: CFStringRef,
        subgroup: CFStringRef,
        a: u64,
        b: u64,
        c: u64,
    ) -> CFDictionaryRef;

    fn IOReportCreateSubscription(
        a: CFTypeRef,
        channels: CFDictionaryRef,
        b: *mut CFMutableDictionaryRef,
        c: u64,
        d: CFTypeRef,
    ) -> IOReportSubscriptionRef;

    fn IOReportCreateSamples(
        subscription: IOReportSubscriptionRef,
        channels: CFMutableDictionaryRef,
        a: CFTypeRef,
    ) -> CFDictionaryRef;

    fn IOReportCreateSamplesDelta(
        prev: CFDictionaryRef,
        current: CFDictionaryRef,
        a: CFTypeRef,
    ) -> CFDictionaryRef;

    fn IOReportChannelGetGroup(channel: CFDictionaryRef) -> CFStringRef;
    fn IOReportChannelGetSubGroup(channel: CFDictionaryRef) -> CFStringRef;
    fn IOReportChannelGetChannelName(channel: CFDictionaryRef) -> CFStringRef;
    fn IOReportSimpleGetIntegerValue(channel: CFDictionaryRef, a: *mut i32) -> i64;

    // IOKit for temperature
    fn IOServiceMatching(name: *const u8) -> CFMutableDictionaryRef;
    fn IOServiceGetMatchingServices(
        mainPort: u32,
        matching: CFMutableDictionaryRef,
        existing: *mut u32,
    ) -> i32;
    fn IOIteratorNext(iterator: u32) -> u32;
    fn IORegistryEntryCreateCFProperties(
        entry: u32,
        properties: *mut CFMutableDictionaryRef,
        allocator: CFAllocatorRef,
        options: u32,
    ) -> i32;
    fn IOObjectRelease(object: u32) -> i32;
}

const kCFStringEncodingUTF8: u32 = 0x08000100;
const kCFNumberSInt64Type: i32 = 4;
const kCFNumberFloat64Type: i32 = 6;

// ─── Helper functions ────────────────────────────────────────────────

unsafe fn cfstr(s: &str) -> CFStringRef {
    let cstr = format!("{s}\0");
    CFStringCreateWithCString(kCFAllocatorDefault, cstr.as_ptr(), kCFStringEncodingUTF8)
}

unsafe fn cfstring_to_string(cf: CFStringRef) -> Option<String> {
    if cf.is_null() {
        return None;
    }
    let len = CFStringGetLength(cf);
    if len <= 0 {
        return None;
    }
    let buf_size = (len * 4 + 1) as usize;
    let mut buf = vec![0u8; buf_size];
    if CFStringGetCString(cf, buf.as_mut_ptr(), buf_size as CFIndex, kCFStringEncodingUTF8) {
        let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        Some(String::from_utf8_lossy(&buf[..end]).to_string())
    } else {
        None
    }
}

unsafe fn cfnumber_to_i64(cf: CFNumberRef) -> Option<i64> {
    if cf.is_null() {
        return None;
    }
    let mut val: i64 = 0;
    if CFNumberGetValue(cf, kCFNumberSInt64Type, &mut val as *mut i64 as *mut c_void) {
        Some(val)
    } else {
        None
    }
}

unsafe fn cfnumber_to_f64(cf: CFNumberRef) -> Option<f64> {
    if cf.is_null() {
        return None;
    }
    let mut val: f64 = 0.0;
    if CFNumberGetValue(cf, kCFNumberFloat64Type, &mut val as *mut f64 as *mut c_void) {
        Some(val)
    } else {
        None
    }
}

// ─── IOReport sampling ───────────────────────────────────────────────

/// Holds the IOReport subscription state between samples
pub struct AppleGpuSampler {
    subscription: IOReportSubscriptionRef,
    channels: CFMutableDictionaryRef,
    prev_sample: Option<CFDictionaryRef>,
}

/// Metrics returned from a single sample delta
pub struct AppleGpuMetrics {
    pub gpu_name: String,
    pub utilization: u32,     // 0-100%
    pub temperature: u32,     // degrees C
    pub power_mw: Option<u32>, // milliwatts
    pub freq_mhz: Option<u32>,
}

impl AppleGpuSampler {
    /// Initialize IOReport subscription for GPU channels.
    /// Returns None if IOReport is unavailable (non-Apple Silicon, etc.)
    pub fn new() -> Option<Self> {
        unsafe {
            let group = cfstr("GPU");
            let channels = IOReportCopyChannelsInGroup(group, ptr::null(), 0, 0, 0);
            CFRelease(group);

            if channels.is_null() {
                return None;
            }

            // Also subscribe to Energy Model channels for power data
            let energy_group = cfstr("Energy Model");
            let energy_channels =
                IOReportCopyChannelsInGroup(energy_group, ptr::null(), 0, 0, 0);
            CFRelease(energy_group);

            let mut sub_channels: CFMutableDictionaryRef = ptr::null_mut();
            let subscription = IOReportCreateSubscription(
                ptr::null(),
                channels,
                &mut sub_channels,
                0,
                ptr::null(),
            );

            CFRelease(channels as CFTypeRef);
            if !energy_channels.is_null() {
                CFRelease(energy_channels as CFTypeRef);
            }

            if subscription.is_null() || sub_channels.is_null() {
                return None;
            }

            Some(Self {
                subscription,
                channels: sub_channels,
                prev_sample: None,
            })
        }
    }

    /// Take a sample and compute delta from previous sample.
    /// First call returns None (needs two samples for a delta).
    pub fn sample(&mut self) -> Option<AppleGpuMetrics> {
        unsafe {
            let current =
                IOReportCreateSamples(self.subscription, self.channels, ptr::null());
            if current.is_null() {
                return None;
            }

            let result = if let Some(prev) = self.prev_sample {
                let delta = IOReportCreateSamplesDelta(prev, current, ptr::null());
                CFRelease(prev);

                if delta.is_null() {
                    None
                } else {
                    let metrics = self.parse_delta(delta);
                    CFRelease(delta);
                    Some(metrics)
                }
            } else {
                None
            };

            self.prev_sample = Some(current);

            // Get temperature separately via IOKit
            let temperature = read_gpu_temperature().unwrap_or(0);

            result.map(|mut m| {
                m.temperature = temperature;
                m
            })
        }
    }

    unsafe fn parse_delta(&self, delta: CFDictionaryRef) -> AppleGpuMetrics {
        let mut gpu_name = String::from("Apple GPU");
        let mut gpu_busy: i64 = 0;
        let mut gpu_total: i64 = 0;
        let mut power_mw: Option<u32> = None;
        let mut freq_mhz: Option<u32> = None;

        // The delta is a CFDictionary with an "IOReportChannels" array
        let channels_key = cfstr("IOReportChannels");
        let channels_array = CFDictionaryGetValue(delta, channels_key);
        CFRelease(channels_key);

        if channels_array.is_null() {
            return AppleGpuMetrics {
                gpu_name,
                utilization: 0,
                temperature: 0,
                power_mw: None,
                freq_mhz: None,
            };
        }

        let count = CFArrayGetCount(channels_array);

        for i in 0..count {
            let channel = CFArrayGetValueAtIndex(channels_array, i);
            if channel.is_null() {
                continue;
            }

            let group = cfstring_to_string(IOReportChannelGetGroup(channel))
                .unwrap_or_default();
            let subgroup = cfstring_to_string(IOReportChannelGetSubGroup(channel))
                .unwrap_or_default();
            let name = cfstring_to_string(IOReportChannelGetChannelName(channel))
                .unwrap_or_default();

            if group == "GPU" && subgroup == "GPU Activity" {
                let val = IOReportSimpleGetIntegerValue(channel, ptr::null_mut());
                if name.contains("Busy") || name.contains("active") {
                    gpu_busy += val;
                }
                // Total includes busy + idle
                gpu_total += val;
            }

            if group == "GPU" && name.contains("Core Clock") {
                let val = IOReportSimpleGetIntegerValue(channel, ptr::null_mut());
                if val > 0 {
                    freq_mhz = Some(val as u32);
                }
            }

            if group == "Energy Model" && name.contains("GPU") {
                let val = IOReportSimpleGetIntegerValue(channel, ptr::null_mut());
                if val > 0 {
                    power_mw = Some(val as u32);
                }
            }

            // Try to get a better GPU name
            if group == "GPU" && gpu_name == "Apple GPU" && !name.is_empty() {
                if name.starts_with("Apple") || name.starts_with("M") {
                    gpu_name = name.clone();
                }
            }
        }

        let utilization = if gpu_total > 0 {
            ((gpu_busy as f64 / gpu_total as f64) * 100.0) as u32
        } else {
            0
        };

        AppleGpuMetrics {
            gpu_name,
            utilization: utilization.min(100),
            temperature: 0, // filled in by caller
            power_mw,
            freq_mhz,
        }
    }
}

impl Drop for AppleGpuSampler {
    fn drop(&mut self) {
        unsafe {
            if let Some(prev) = self.prev_sample {
                CFRelease(prev);
            }
            // subscription and channels are managed by IOReport
        }
    }
}

// ─── Temperature via IOKit ───────────────────────────────────────────

/// Read GPU temperature from AppleSMC via IOKit.
/// Falls back to 0 if unavailable.
fn read_gpu_temperature() -> Option<u32> {
    unsafe {
        let matching = IOServiceMatching(b"AppleARMIODevice\0".as_ptr());
        if matching.is_null() {
            return None;
        }

        let mut iterator: u32 = 0;
        let kr = IOServiceGetMatchingServices(0, matching, &mut iterator);
        if kr != 0 {
            return None;
        }

        let mut temp: Option<u32> = None;
        loop {
            let entry = IOIteratorNext(iterator);
            if entry == 0 {
                break;
            }

            let mut props: CFMutableDictionaryRef = ptr::null_mut();
            if IORegistryEntryCreateCFProperties(entry, &mut props, kCFAllocatorDefault, 0) == 0
                && !props.is_null()
            {
                let key = cfstr("temperature");
                let val = CFDictionaryGetValue(props as CFDictionaryRef, key);
                CFRelease(key);

                if !val.is_null() {
                    if let Some(t) = cfnumber_to_i64(val) {
                        // Temperature is in centi-degrees or direct degrees depending on sensor
                        let degrees = if t > 1000 { t / 100 } else { t };
                        if degrees > 0 && degrees < 150 {
                            temp = Some(degrees as u32);
                        }
                    }
                }
                CFRelease(props as CFTypeRef);
            }
            IOObjectRelease(entry);

            if temp.is_some() {
                break;
            }
        }
        IOObjectRelease(iterator);
        temp
    }
}

/// Get GPU name from system_profiler (more reliable than IOReport for the name)
pub fn get_apple_gpu_name() -> String {
    use std::process::Command;
    let output = Command::new("system_profiler")
        .args(["SPDisplaysDataType", "-json"])
        .output()
        .ok();

    if let Some(out) = output {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.contains("sppci_model") {
                if let Some(name) = trimmed.split('"').nth(3) {
                    return name.to_string();
                }
            }
        }
    }

    "Apple GPU".to_string()
}
