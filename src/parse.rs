use std::path::Path;

use anyhow::{Result, anyhow};

use crate::counts::{Count, MixedCounts, MixedKey, SingleCounts, add_count};
use crate::profile::{Lib, Samples, Thread};

pub(crate) fn parse_single_image_profile(
    profile: &crate::profile::Profile,
    binary: &Path,
    cpu_weighted: bool,
) -> Result<SingleCounts> {
    let target_libs = build_target_lib_flags(&profile.libs, binary);
    let mut counts = SingleCounts::default();

    for thread in &profile.threads {
        if thread.samples.stack.is_empty() {
            continue;
        }

        let frame_names = build_frame_names(thread)?;
        let target_frames = build_target_frame_flags(thread, &target_libs)?;
        let weights = SampleWeights::new(&thread.samples, cpu_weighted);

        for (sample_idx, stack_idx) in thread.samples.stack.iter().copied().enumerate() {
            let Some(mut idx) = stack_idx else {
                continue;
            };
            let weight = weights.get(sample_idx);
            let mut leaf_raw = None;

            loop {
                let frame_idx = frame_index(thread, idx)?;
                if target_frames.get(frame_idx).copied().unwrap_or(false) {
                    let raw = frame_names[frame_idx].clone();
                    if leaf_raw.is_none() {
                        leaf_raw = Some(raw.clone());
                    }
                    add_count(&mut counts.all, raw.clone(), weight);
                    add_count(&mut counts.addr, raw, weight);
                }

                match stack_prefix(thread, idx) {
                    Some(next) => idx = next,
                    None => break,
                }
            }

            if let Some(raw) = leaf_raw {
                add_count(&mut counts.leaf, raw, weight);
            }
        }
    }

    Ok(counts)
}

pub(crate) fn parse_mixed_image_profile(
    profile: &crate::profile::Profile,
    cpu_weighted: bool,
) -> Result<MixedCounts> {
    let mut counts = MixedCounts::default();

    for thread in &profile.threads {
        if thread.samples.stack.is_empty() {
            continue;
        }

        let frame_keys = build_frame_keys(thread, &profile.libs)?;
        let weights = SampleWeights::new(&thread.samples, cpu_weighted);

        for (sample_idx, stack_idx) in thread.samples.stack.iter().copied().enumerate() {
            let Some(mut idx) = stack_idx else {
                continue;
            };
            let weight = weights.get(sample_idx);

            let frame_idx = frame_index(thread, idx)?;
            add_count(&mut counts.leaf, frame_keys[frame_idx].clone(), weight);

            loop {
                let frame_idx = frame_index(thread, idx)?;
                add_count(&mut counts.all, frame_keys[frame_idx].clone(), weight);

                match stack_prefix(thread, idx) {
                    Some(next) => idx = next,
                    None => break,
                }
            }
        }
    }

    Ok(counts)
}

struct SampleWeights<'a> {
    deltas: Option<&'a [serde_json::Number]>,
}

impl<'a> SampleWeights<'a> {
    fn new(samples: &'a Samples, cpu_weighted: bool) -> Self {
        let deltas = cpu_weighted
            .then_some(samples.thread_cpu_delta.as_deref())
            .flatten()
            .filter(|deltas| deltas.len() == samples.stack.len());
        Self { deltas }
    }

    fn get(&self, idx: usize) -> Count {
        let Some(delta) = self.deltas.and_then(|deltas| deltas.get(idx)) else {
            return 1;
        };

        delta
            .as_u64()
            .or_else(|| delta.as_i64().and_then(|value| Count::try_from(value).ok()))
            .or_else(|| delta.as_f64().map(|value| value.max(0.0).round() as Count))
            .unwrap_or(0)
    }
}

fn build_frame_names(thread: &Thread) -> Result<Vec<String>> {
    (0..thread.frame_table.func.len())
        .map(|frame_idx| raw_name(thread, frame_idx))
        .collect()
}

fn build_frame_keys(thread: &Thread, libs: &[Lib]) -> Result<Vec<MixedKey>> {
    (0..thread.frame_table.func.len())
        .map(|frame_idx| {
            let raw = raw_name(thread, frame_idx)?;
            let image = lib_index_for_frame(thread, frame_idx)?
                .and_then(|lib_idx| libs.get(lib_idx))
                .and_then(|lib| lib.path.clone());
            Ok(MixedKey { image, raw })
        })
        .collect()
}

fn build_target_frame_flags(thread: &Thread, target_libs: &[bool]) -> Result<Vec<bool>> {
    (0..thread.frame_table.func.len())
        .map(|frame_idx| {
            if thread.func_table.resource.is_none() {
                return Ok(true);
            }

            let is_target = lib_index_for_frame(thread, frame_idx)?
                .and_then(|lib_idx| target_libs.get(lib_idx).copied())
                .unwrap_or(false);
            Ok(is_target)
        })
        .collect()
}

fn frame_index(thread: &Thread, stack_idx: usize) -> Result<usize> {
    thread
        .stack_table
        .frame
        .get(stack_idx)
        .copied()
        .ok_or_else(|| anyhow!("stack frame index {stack_idx} out of bounds"))
}

fn stack_prefix(thread: &Thread, stack_idx: usize) -> Option<usize> {
    thread.stack_table.prefix.get(stack_idx).copied().flatten()
}

fn raw_name(thread: &Thread, frame_idx: usize) -> Result<String> {
    let func_idx = thread
        .frame_table
        .func
        .get(frame_idx)
        .copied()
        .ok_or_else(|| anyhow!("frame func index for frame {frame_idx} out of bounds"))?;
    let name_idx = thread
        .func_table
        .name
        .get(func_idx)
        .copied()
        .ok_or_else(|| anyhow!("function name index for function {func_idx} out of bounds"))?;

    Ok(thread
        .string_array
        .get(name_idx)
        .cloned()
        .unwrap_or_else(|| format!("<{name_idx}>")))
}

fn lib_index_for_frame(thread: &Thread, frame_idx: usize) -> Result<Option<usize>> {
    let Some(resources) = &thread.func_table.resource else {
        return Ok(None);
    };

    let func_idx = thread
        .frame_table
        .func
        .get(frame_idx)
        .copied()
        .ok_or_else(|| anyhow!("frame func index for frame {frame_idx} out of bounds"))?;
    let Some(Some(resource_idx)) = resources.get(func_idx).copied() else {
        return Ok(None);
    };
    if resource_idx < 0 {
        return Ok(None);
    }

    let Some(Some(lib_idx)) = thread
        .resource_table
        .lib
        .get(resource_idx as usize)
        .copied()
    else {
        return Ok(None);
    };
    if lib_idx < 0 {
        return Ok(None);
    }

    Ok(Some(lib_idx as usize))
}

fn build_target_lib_flags(libs: &[Lib], binary: &Path) -> Vec<bool> {
    let binary_canonical = binary.canonicalize().ok();
    libs.iter()
        .map(|lib| {
            lib.path
                .as_deref()
                .map(|image_path| {
                    image_matches_binary_cached(image_path, binary, binary_canonical.as_deref())
                })
                .unwrap_or(false)
        })
        .collect()
}

fn image_matches_binary_cached(
    image_path: &str,
    binary: &Path,
    binary_canonical: Option<&Path>,
) -> bool {
    let image = Path::new(image_path);
    if image == binary {
        return true;
    }
    if image.file_name() == binary.file_name() {
        return true;
    }

    match (image.canonicalize(), binary_canonical) {
        (Ok(image), Some(binary)) => image == binary,
        _ => false,
    }
}
