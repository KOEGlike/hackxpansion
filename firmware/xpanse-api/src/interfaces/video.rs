//! Direct RGB565 video resources available to apps.

use alloc::sync::Arc;
use core::{
    cell::RefCell,
    marker::PhantomData,
    sync::atomic::{AtomicU32, Ordering},
};

use embassy_sync::blocking_mutex::{CriticalSectionMutex, ThreadModeMutex};

pub use slint::platform::software_renderer::Rgb565Pixel;

static NEXT_FRAME_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, defmt::Format)]
pub enum FrameBufferError {
    InvalidDimensions,
}

struct FrameStorage {
    pixels: ThreadModeMutex<RefCell<&'static mut [Rgb565Pixel]>>,
    len: usize,
    width: u16,
    height: u16,
}

struct ActiveFrame {
    id: u32,
    width: u16,
    height: u16,
    revision: AtomicU32,
}

struct FrameLink {
    storage: FrameStorage,
    active: CriticalSectionMutex<RefCell<Option<Arc<ActiveFrame>>>>,
}

/// The app-facing RGB565 framebuffer capability stored in the registry.
pub struct Rgb565FrameBuffer {
    link: Arc<FrameLink>,
}

/// The display-task endpoint paired with [`Rgb565FrameBuffer`].
pub struct Rgb565FrameDisplay {
    link: Arc<FrameLink>,
}

/// Creates a platform framebuffer resource backed by caller-provided storage.
pub fn rgb565_frame_buffer(
    pixels: &'static mut [Rgb565Pixel],
    width: u16,
    height: u16,
) -> Result<(Rgb565FrameBuffer, Rgb565FrameDisplay), FrameBufferError> {
    let expected_len = usize::from(width)
        .checked_mul(usize::from(height))
        .filter(|len| *len > 0)
        .ok_or(FrameBufferError::InvalidDimensions)?;
    if pixels.len() != expected_len {
        return Err(FrameBufferError::InvalidDimensions);
    }

    let len = pixels.len();
    let link = Arc::new(FrameLink {
        storage: FrameStorage {
            pixels: ThreadModeMutex::new(RefCell::new(pixels)),
            len,
            width,
            height,
        },
        active: CriticalSectionMutex::new(RefCell::new(None)),
    });
    Ok((
        Rgb565FrameBuffer {
            link: Arc::clone(&link),
        },
        Rgb565FrameDisplay { link },
    ))
}

impl Rgb565FrameBuffer {
    /// Starts a direct-rendering session which remains active until dropped.
    pub fn start(
        &mut self,
        width: u16,
        height: u16,
    ) -> Result<Rgb565FrameSession<'_>, FrameBufferError> {
        let len = usize::from(width)
            .checked_mul(usize::from(height))
            .filter(|len| *len > 0)
            .ok_or(FrameBufferError::InvalidDimensions)?;
        if width > self.link.storage.width
            || height > self.link.storage.height
            || len > self.link.storage.len
        {
            return Err(FrameBufferError::InvalidDimensions);
        }

        let frame = Arc::new(ActiveFrame {
            id: NEXT_FRAME_ID.fetch_add(1, Ordering::Relaxed),
            width,
            height,
            revision: AtomicU32::new(0),
        });
        self.link
            .storage
            .pixels
            .lock(|pixels| pixels.borrow_mut()[..len].fill(Rgb565Pixel(0)));
        self.link
            .active
            .lock(|active| *active.borrow_mut() = Some(Arc::clone(&frame)));

        Ok(Rgb565FrameSession {
            link: Arc::clone(&self.link),
            frame,
            resource: PhantomData,
        })
    }
}

/// A scoped direct-rendering session borrowed from the registry resource.
pub struct Rgb565FrameSession<'a> {
    link: Arc<FrameLink>,
    frame: Arc<ActiveFrame>,
    resource: PhantomData<&'a mut Rgb565FrameBuffer>,
}

impl Rgb565FrameSession<'_> {
    pub fn set_pixel(&self, x: u16, y: u16, color: Rgb565Pixel) {
        if x >= self.frame.width || y >= self.frame.height {
            return;
        }
        let index = usize::from(y) * usize::from(self.frame.width) + usize::from(x);
        self.link
            .storage
            .pixels
            .lock(|pixels| pixels.borrow_mut()[index] = color);
    }

    /// Makes all pixel writes since the previous call available to the display task.
    pub fn present(&self) {
        self.frame.revision.fetch_add(1, Ordering::Release);
    }
}

impl Drop for Rgb565FrameSession<'_> {
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
    link: Arc<FrameLink>,
    frame: Arc<ActiveFrame>,
}

impl PresentedFrame {
    pub fn width(&self) -> u16 {
        self.frame.width
    }

    pub fn height(&self) -> u16 {
        self.frame.height
    }

    pub fn len(&self) -> usize {
        usize::from(self.frame.width) * usize::from(self.frame.height)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn with_pixels<R>(&self, f: impl FnOnce(&[Rgb565Pixel]) -> R) -> R {
        let len = self.len();
        self.link
            .storage
            .pixels
            .lock(|pixels| f(&pixels.borrow()[..len]))
    }

    pub fn token(&self) -> u64 {
        let revision = self.frame.revision.load(Ordering::Acquire);
        (u64::from(self.frame.id) << 32) | u64::from(revision)
    }
}

impl Rgb565FrameDisplay {
    /// Returns the frame currently presented by the app holding the resource.
    pub fn active_frame(&self) -> Option<PresentedFrame> {
        self.link.active.lock(|active| {
            active
                .borrow()
                .as_ref()
                .cloned()
                .map(|frame| PresentedFrame {
                    link: Arc::clone(&self.link),
                    frame,
                })
        })
    }

    /// Gives the platform mutable access to the shared storage while direct video is inactive.
    pub fn with_buffer_mut<R>(&self, f: impl FnOnce(&mut [Rgb565Pixel]) -> R) -> Option<R> {
        if self.link.active.lock(|active| active.borrow().is_some()) {
            return None;
        }
        self.link.storage.pixels.lock(|pixels| {
            let mut pixels = pixels.borrow_mut();
            Some(f(&mut pixels))
        })
    }
}
