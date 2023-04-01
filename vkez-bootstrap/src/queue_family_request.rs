use std::{borrow::Cow, cmp::Ordering};

use ash::vk;
use vkez_core::ash;

#[derive(Default, Debug, Clone)]
pub struct QueueFamilyRequest {
    must_support: vk::QueueFlags,
    prefer_support: vk::QueueFlags,
    must_not_support: vk::QueueFlags,
    prefer_not_support: vk::QueueFlags,
    priorities: Vec<f32>,
}

impl From<QueueFamilyRequest> for Cow<'static, QueueFamilyRequest> {
    fn from(value: QueueFamilyRequest) -> Self {
        Self::Owned(value)
    }
}

impl<'a> From<&'a QueueFamilyRequest> for Cow<'a, QueueFamilyRequest> {
    fn from(value: &'a QueueFamilyRequest) -> Self {
        Self::Borrowed(value)
    }
}

impl QueueFamilyRequest {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn require_graphics(mut self) -> Self {
        self.must_support |= vk::QueueFlags::GRAPHICS;
        self
    }

    pub fn require_compute(mut self) -> Self {
        self.must_support |= vk::QueueFlags::COMPUTE;
        self
    }

    pub fn require_transfer(mut self) -> Self {
        self.must_support |= vk::QueueFlags::TRANSFER;
        self
    }

    pub fn prefer_alone(mut self) -> Self {
        self.prefer_not_support |= !(self.must_support | self.prefer_support);
        self
    }

    pub fn must_support(mut self, must_support: vk::QueueFlags) -> Self {
        self.must_support |= must_support;
        self
    }

    pub fn prefer_support(mut self, prefer_support: vk::QueueFlags) -> Self {
        self.prefer_support |= prefer_support;
        self
    }

    pub fn must_not_support(mut self, must_not_support: vk::QueueFlags) -> Self {
        self.must_not_support |= must_not_support;
        self
    }

    pub fn prefer_not_support(mut self, prefer_not_support: vk::QueueFlags) -> Self {
        self.prefer_not_support |= prefer_not_support;
        self
    }

    pub fn amount_with_priorities(mut self, priorities: impl Into<Vec<f32>>) -> Self {
        self.priorities = priorities.into();
        self
    }

    pub fn amount(mut self, amount: usize) -> Self {
        self.priorities = vec![1.0; amount];
        self
    }

    pub fn choose_queue_family_index(&self, queues: &[vk::QueueFamilyProperties]) -> Option<u32> {
        let supported = queues
            .iter()
            .enumerate()
            .filter(|(_, q)| q.queue_flags.contains(self.must_support))
            .filter(|(_, q)| !q.queue_flags.intersects(self.must_not_support))
            .filter(|(_, q)| q.queue_count as usize >= self.priorities.len());

        supported
            .fold(None, |best, (i, queue)| {
                let extra_supports = (queue.queue_flags & self.prefer_support)
                    .as_raw()
                    .count_ones();
                let extra_excludes = (queue.queue_flags & self.prefer_not_support)
                    .as_raw()
                    .count_ones();

                // Take the queue with the most prefer support
                match best {
                    None => Some((i, queue, extra_supports, extra_excludes)),
                    Some(best @ (_, _, best_extra_support, best_extra_excludes)) => {
                        match (
                            extra_supports.cmp(&best_extra_support),
                            extra_excludes.cmp(&best_extra_excludes),
                        ) {
                            (Ordering::Greater, _) => {
                                Some((i, queue, extra_supports, extra_excludes))
                            }
                            (Ordering::Less, _) => Some(best),
                            (Ordering::Equal, Ordering::Greater) => {
                                Some((i, queue, extra_supports, extra_excludes))
                            }
                            (Ordering::Equal, _) => Some(best),
                        }
                    }
                }
            })
            .map(|(index, _, _, _)| index as u32)
    }

    pub fn get_create_info(
        &self, queues: &[vk::QueueFamilyProperties],
    ) -> Option<vk::DeviceQueueCreateInfoBuilder> {
        let Some(index) = self.choose_queue_family_index(queues) else {
            return None;
        };

        Some(
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(index as _)
                .queue_priorities(&self.priorities),
        )
    }
}
