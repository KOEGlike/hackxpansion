//! Direct indexed-color video resources available to apps.

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{
    cell::RefCell,
    marker::PhantomData,
    sync::atomic::{AtomicU8, AtomicU32, Ordering},
};

use embassy_sync::blocking_mutex::CriticalSectionMutex;

static NEXT_FRAME_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, defmt::Format)]
pub enum FrameBufferError {
    InvalidDimensions,
    EmptyPalette,
}

struct IndexedFrame {
    id: u32,
    width: u16,
    height: u16,
    pixels: Box<[AtomicU8]>,
    palette: &'static [u16],
    revision: AtomicU32,
}

struct FrameLink {
    active: CriticalSectionMutex<RefCell<Option<Arc<IndexedFrame>>>>,
}

/// The app-facing indexed-color framebuffer capability stored in the registry.
pub struct IndexedFrameBuffer {
    link: Arc<FrameLink>,
}

/// The display-task endpoint paired with [`IndexedFrameBuffer`].
pub struct IndexedFrameDisplay {
    link: Arc<FrameLink>,
}

/// Creates the platform framebuffer resource and its display-task endpoint.
pub fn indexed_frame_buffer() -> (IndexedFrameBuffer, IndexedFrameDisplay) {
    let link = Arc::new(FrameLink {
        active: CriticalSectionMutex::new(RefCell::new(None)),
    });
    (
        IndexedFrameBuffer {
            link: Arc::clone(&link),
        },
        IndexedFrameDisplay { link },
    )
}

impl IndexedFrameBuffer {
    /// Starts a direct-rendering session which remains active until dropped.
    pub fn start(
        &mut self,
        width: u16,
        height: u16,
        palette: &'static [u16],
    ) -> Result<IndexedFrameSession<'_>, FrameBufferError> {
        let len = usize::from(width)
            .checked_mul(usize::from(height))
            .filter(|len| *len > 0)
            .ok_or(FrameBufferError::InvalidDimensions)?;
        if palette.is_empty() {
            return Err(FrameBufferError::EmptyPalette);
        }

        let pixels = (0..len)
            .map(|_| AtomicU8::new(0))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let frame = Arc::new(IndexedFrame {
            id: NEXT_FRAME_ID.fetch_add(1, Ordering::Relaxed),
            width,
            height,
            pixels,
            palette,
            revision: AtomicU32::new(0),
        });
        self.link
            .active
            .lock(|active| *active.borrow_mut() = Some(frame.clone()));

        Ok(IndexedFrameSession {
            link: self.link.clone(),
            frame,
            resource: PhantomData,
        })
    }
}

/// A scoped direct-rendering session borrowed from the registry resource.
pub struct IndexedFrameSession<'a> {
    link: Arc<FrameLink>,
    frame: Arc<IndexedFrame>,
    resource: PhantomData<&'a mut IndexedFrameBuffer>,
}

impl IndexedFrameSession<'_> {
    pub fn set_pixel(&self, x: u16, y: u16, color: u8) {
        if x >= self.frame.width || y >= self.frame.height {
            return;
        }
        let index = usize::from(y) * usize::from(self.frame.width) + usize::from(x);
        self.frame.pixels[index].store(color, Ordering::Relaxed);
    }

    /// Makes all pixel writes since the previous call available to the display task.
    pub fn present(&self) {
        self.frame.revision.fetch_add(1, Ordering::Release);
    }
}

impl Drop for IndexedFrameSession<'_> {
    fn drop(&mut self) {
        self.link.active.lock(|active| {
            let mut active = active.borrow_mut();
            if active
                .as_ref()
                .is_some_and(|frame| Arc::ptr_eq(frame, &self.frame))
            {
                *active = None;
            }
        });
    }
}

pub struct PresentedFrame {
    frame: Arc<IndexedFrame>,
}

impl PresentedFrame {
    pub fn width(&self) -> u16 {
        self.frame.width
    }

    pub fn height(&self) -> u16 {
        self.frame.height
    }

    pub fn len(&self) -> usize {
        self.frame.pixels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frame.pixels.is_empty()
    }

    pub fn pixel(&self, index: usize) -> u8 {
        self.frame.pixels[index].load(Ordering::Relaxed)
    }

    pub fn color(&self, index: u8) -> u16 {
        self.frame
            .palette
            .get(usize::from(index))
            .copied()
            .unwrap_or(0)
    }

    pub fn token(&self) -> u64 {
        let revision = self.frame.revision.load(Ordering::Acquire);
        (u64::from(self.frame.id) << 32) | u64::from(revision)
    }
}

impl IndexedFrameDisplay {
    /// Returns the frame currently presented by the app holding the resource.
    pub fn active_frame(&self) -> Option<PresentedFrame> {
        self.link.active.lock(|active| {
            active
                .borrow()
                .as_ref()
                .cloned()
                .map(|frame| PresentedFrame { frame })
        })
    }
}
