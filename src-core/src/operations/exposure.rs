use crate::graph::{SceneManager, ElementId, FrameNumber, DrawingId, ExposureBlock};

pub fn set_exposure(scene: &mut SceneManager, element_id: ElementId, start_frame: FrameNumber, duration: u32, drawing_id: DrawingId) {
    if duration == 0 { return; }
    if let Some(el) = scene.elements.get_mut(&element_id) {
        let end_frame = start_frame + duration;
        let mut to_insert = Vec::new();
        let mut to_remove = Vec::new();

        // 1. Check if we are slicing a block that already exists just before our insert point
        if let Some((&k, prev_block)) = el.exposures.range(..=start_frame).next_back() {
            let prev_end = prev_block.start_frame + prev_block.duration;
            if prev_end > start_frame {
                let new_duration = start_frame - prev_block.start_frame;
                if new_duration > 0 {
                    to_insert.push((k, ExposureBlock { duration: new_duration, ..*prev_block }));
                } else {
                    to_remove.push(k);
                }
                // If it extended PAST our new block, slice the tail end
                if prev_end > end_frame {
                    to_insert.push((end_frame, ExposureBlock { drawing_id: prev_block.drawing_id, start_frame: end_frame, duration: prev_end - end_frame }));
                }
            }
        }

        // 2. Remove any blocks completely swallowed by our new block
        let overlapping: Vec<_> = el.exposures.range(start_frame..end_frame).map(|(&k, b)| (k, b.clone())).collect();
        for (k, block) in overlapping {
            to_remove.push(k);
            let block_end = block.start_frame + block.duration;
            if block_end > end_frame {
                to_insert.push((end_frame, ExposureBlock { drawing_id: block.drawing_id, start_frame: end_frame, duration: block_end - end_frame }));
            }
        }

        for k in to_remove { el.exposures.remove(&k); }
        for (k, b) in to_insert { el.exposures.insert(k, b); }

        // 3. Insert the final block
        el.exposures.insert(start_frame, ExposureBlock { drawing_id, start_frame, duration });
    }
}

pub fn clear_exposure(scene: &mut SceneManager, element_id: ElementId, start_frame: FrameNumber, duration: u32) {
    if duration == 0 { return; }
    if let Some(el) = scene.elements.get_mut(&element_id) {
        let end_frame = start_frame + duration;
        let mut to_insert = Vec::new();
        let mut to_remove = Vec::new();

        if let Some((&k, prev_block)) = el.exposures.range(..=start_frame).next_back() {
            let prev_end = prev_block.start_frame + prev_block.duration;
            if prev_end > start_frame {
                let new_duration = start_frame - prev_block.start_frame;
                if new_duration > 0 { to_insert.push((k, ExposureBlock { duration: new_duration, ..*prev_block })); } else { to_remove.push(k); }
                if prev_end > end_frame { to_insert.push((end_frame, ExposureBlock { drawing_id: prev_block.drawing_id, start_frame: end_frame, duration: prev_end - end_frame })); }
            }
        }

        let overlapping: Vec<_> = el.exposures.range(start_frame..end_frame).map(|(&k, b)| (k, b.clone())).collect();
        for (k, block) in overlapping {
            to_remove.push(k);
            let block_end = block.start_frame + block.duration;
            if block_end > end_frame { to_insert.push((end_frame, ExposureBlock { drawing_id: block.drawing_id, start_frame: end_frame, duration: block_end - end_frame })); }
        }

        for k in to_remove { el.exposures.remove(&k); }
        for (k, b) in to_insert { el.exposures.insert(k, b); }
    }
}

pub fn extend_exposure(scene: &mut SceneManager, element_id: ElementId, start_frame: FrameNumber, new_duration: u32) {
    let mut root_block = None;
    if let Some(el) = scene.elements.get(&element_id) {
        if let Some(block) = el.get_exposure_at(start_frame) {
            root_block = Some((block.start_frame, block.drawing_id));
        }
    }
    if let Some((block_start, drawing_id)) = root_block {
        set_exposure(scene, element_id, block_start, new_duration, drawing_id);
    }
}

pub fn split_exposure(scene: &mut SceneManager, element_id: ElementId, cut_frame: FrameNumber) {
    if let Some(el) = scene.elements.get_mut(&element_id) {
        if let Some((&k, block)) = el.exposures.range(..=cut_frame).next_back() {
            let end = block.start_frame + block.duration;
            if cut_frame > block.start_frame && cut_frame < end {
                let drawing_id = block.drawing_id;
                let first_dur = cut_frame - block.start_frame;
                let second_dur = end - cut_frame;
                let mut b1 = block.clone(); b1.duration = first_dur;
                el.exposures.insert(k, b1);
                el.exposures.insert(cut_frame, ExposureBlock { drawing_id, start_frame: cut_frame, duration: second_dur });
            }
        }
    }
}